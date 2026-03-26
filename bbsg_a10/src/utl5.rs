use crate::dcl::PeaAssVar;
use crate::dcl::VarType;
use crate::dcl::FIR_FLDS;
use rust_xlsxwriter::*;
use std::error::Error;

const DATA_FLDS: [crate::dcl::VarType; 12] = [
    VarType::NoMet1Ph,
    VarType::NoMet3Ph,
    VarType::NoPeaTr,
    VarType::BessMWh,
    VarType::TpoAdd,
    VarType::SvgAdd,
    VarType::EcuAdd,
    VarType::NoMet1PhPlc,
    VarType::NoMet3PhPlc,
    VarType::NoPeaTrPlc,
    VarType::NoMet1PhA,
    VarType::NoMet3PhA,
];

pub fn excel_sub_repo1(xlsx: &str, mxrw: Option<usize>) -> Result<(), Box<dyn Error>> {
    println!("cmd1");
    let mut workbook = Workbook::new();
    let mut flds = vec![VarType::FirSum, VarType::EirSum];
    let mut fld1 = FIR_FLDS.to_vec();
    flds.append(&mut fld1);
    let dnm = crate::dcl::get_dirnm();
    let Ok(buf) = std::fs::read(format!("{dnm}/000-sbrw.bin")) else {
        return Err("Not found".into());
    };
    // ==== read rw3 data
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    assv0.sort_by(|a, b| {
        let a0 = a.v[VarType::Uc1Rank.tousz()].v
            + a.v[VarType::Uc2Rank.tousz()].v
            + a.v[VarType::Uc3Rank.tousz()].v;
        let b0 = b.v[VarType::Uc1Rank.tousz()].v
            + b.v[VarType::Uc2Rank.tousz()].v
            + b.v[VarType::Uc3Rank.tousz()].v;
        a0.partial_cmp(&b0).unwrap()
    });

    let sht = workbook.add_worksheet();
    let _ = sht.set_name("ปริมาณงาน");
    sht.set_column_width(0, 5)?;
    sht.set_column_width(1, 20)?;
    let ttfm = Format::new().set_bold();
    let hdfm = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xC6EFCE));
    let mut rw = 1;
    let datanm = "ปริมาณงาน".to_string();
    let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
    rw += 2;
    let nos = "ลำดับ".to_string();
    let prv = "subst".to_string();
    let _ = sht.write_with_format(rw, 0, nos, &hdfm);
    let _ = sht.write_with_format(rw, 1, prv, &hdfm);
    for (i, co) in DATA_FLDS.iter().enumerate() {
        let vas = format!("{co:?}");
        let _ = sht.write_with_format(rw, i as u16 + 2, vas, &hdfm);
    }
    let mut fmts = vec![];
    for dt in DATA_FLDS.iter() {
        let dtn = format!("{dt:?}");
        if dtn.ends_with("MWh") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else {
            fmts.push(Format::new().set_num_format("#,##0"));
        }
    }
    for (i, ass) in assv0.iter().enumerate() {
        if let Some(mxrw) = mxrw && i>=mxrw {
            break;
        }
        rw += 1;
        let mut co = 0;
        let no = i as i32 + 1;
        let _ = sht.write(rw, co, no);
        co += 1;
        let _ = sht.write(rw, co, ass.sbid.clone());
        for (i, (co, fm)) in DATA_FLDS.iter().zip(fmts.iter()).enumerate() {
            let dt = ass.v[co.tousz()].v;
            let _ = sht.write_with_format(rw, i as u16 + 2, dt, fm)?;
        }
    }

    for f in flds.iter() {
        println!("{f:?}");
        let shtnm = format!("{f:?}");
        let sht = workbook.add_worksheet();
        let _ = sht.set_name(&shtnm);
        sht.set_column_width(0, 5)?;
        sht.set_column_width(1, 20)?;
        let ttfm = Format::new().set_bold();
        let defm = Format::new().set_num_format("#,##0");
        let hdfm = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xC6EFCE));
        for i in 0..15 {
            sht.set_column_width(2 + i, 15)?;
        }
        let mut rw = 1;
        let datanm = format!("ชื่อข้อมูล: {f:?}");
        let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
        rw += 2;
        let nos = "ลำดับ".to_string();
        let prv = "subst".to_string();
        let _ = sht.write_with_format(rw, 0, nos, &hdfm);
        let _ = sht.write_with_format(rw, 1, prv, &hdfm);
        for i in 0..15 {
            let vas = format!("ปี{}", i + 2026);
            let _ = sht.write_with_format(rw, i + 2, vas, &hdfm);
        }

        for (i, ass) in assv0.iter().enumerate() {
            if let Some(mxrw) = mxrw && i>=mxrw {
                break;
            }
            rw += 1;
            let mut co = 0;
            let no = i as i32 + 1;
            let _ = sht.write(rw, co, no);
            co += 1;
            let _ = sht.write(rw, co, ass.sbid.clone());
            for v in ass.vy[f.tousz()].iter() {
                co += 1;
                let _ = sht.write_with_format(rw, co, *v, &defm)?;
            }
        }
    }

    println!("SAVE {} in {xlsx}", assv0.len());
    workbook.save(xlsx)?;
    Ok(())
}

pub fn excel_prv_repo1(xlsx: &str, mxrw: Option<usize>) -> Result<(), Box<dyn Error>> {
    println!("prv {mxrw:?}");
    let mut workbook = Workbook::new();
    let mut flds = vec![VarType::FirSum, VarType::EirSum];
    let mut fld1 = FIR_FLDS.to_vec();
    flds.append(&mut fld1);
    let dnm = crate::dcl::get_dirnm();
    let Ok(buf) = std::fs::read(format!("{dnm}/000-pvrw.bin")) else {
        return Err("Not found".into());
    };
    // ==== read rw3 data
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    assv0.sort_by(|a, b| {
        let a0 = a.v[VarType::Uc1Rank.tousz()].v
            + a.v[VarType::Uc2Rank.tousz()].v
            + a.v[VarType::Uc3Rank.tousz()].v;
        let b0 = b.v[VarType::Uc1Rank.tousz()].v
            + b.v[VarType::Uc2Rank.tousz()].v
            + b.v[VarType::Uc3Rank.tousz()].v;
        a0.partial_cmp(&b0).unwrap()
    });

    let sht = workbook.add_worksheet();
    let _ = sht.set_name("ปริมาณงาน");
    sht.set_column_width(0, 5)?;
    sht.set_column_width(1, 20)?;
    let ttfm = Format::new().set_bold();
    let hdfm = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xC6EFCE));
    let mut rw = 1;
    let datanm = "ปริมาณงาน".to_string();
    let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
    rw += 2;
    let nos = "ลำดับ".to_string();
    let prv = "จังหวัด".to_string();
    let _ = sht.write_with_format(rw, 0, nos, &hdfm);
    let _ = sht.write_with_format(rw, 1, prv, &hdfm);
    for (i, co) in DATA_FLDS.iter().enumerate() {
        let vas = format!("{co:?}");
        let _ = sht.write_with_format(rw, i as u16 + 2, vas, &hdfm);
    }
    let mut fmts = vec![];
    for dt in DATA_FLDS.iter() {
        let dtn = format!("{dt:?}");
        if dtn.ends_with("MWh") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else {
            fmts.push(Format::new().set_num_format("#,##0"));
        }
    }
    for (i, ass) in assv0.iter().enumerate() {
        if let Some(mxrw) = mxrw && i>=mxrw {
            break;
        }
        rw += 1;
        let mut co = 0;
        let no = i as i32 + 1;
        let _ = sht.write(rw, co, no);
        co += 1;
        let _ = sht.write(rw, co, ass.pvid.clone());
        for (i, (co, fm)) in DATA_FLDS.iter().zip(fmts.iter()).enumerate() {
            let dt = ass.v[co.tousz()].v;
            let _ = sht.write_with_format(rw, i as u16 + 2, dt, fm)?;
        }
    }

    for f in flds.iter() {
        println!("{f:?}");
        let shtnm = format!("{f:?}");
        let sht = workbook.add_worksheet();
        let _ = sht.set_name(&shtnm);
        sht.set_column_width(0, 5)?;
        sht.set_column_width(1, 20)?;
        let ttfm = Format::new().set_bold();
        let defm = Format::new().set_num_format("#,##0");
        let hdfm = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xC6EFCE));
        for i in 0..15 {
            sht.set_column_width(2 + i, 15)?;
        }
        let mut rw = 1;
        let datanm = format!("ชื่อข้อมูล: {f:?}");
        let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
        rw += 2;
        let nos = "ลำดับ".to_string();
        let prv = "จังหวัด".to_string();
        let _ = sht.write_with_format(rw, 0, nos, &hdfm);
        let _ = sht.write_with_format(rw, 1, prv, &hdfm);
        for i in 0..15 {
            let vas = format!("ปี{}", i + 2026);
            let _ = sht.write_with_format(rw, i + 2, vas, &hdfm);
        }

        for (i, ass) in assv0.iter().enumerate() {
            if let Some(mxrw) = mxrw && i>=mxrw {
                break;
            }
            rw += 1;
            let mut co = 0;
            let no = i as i32 + 1;
            let _ = sht.write(rw, co, no);
            co += 1;
            let _ = sht.write(rw, co, ass.pvid.clone());
            for v in ass.vy[f.tousz()].iter() {
                co += 1;
                let _ = sht.write_with_format(rw, co, *v, &defm)?;
            }
        }
    }

    println!("SAVE {} in {xlsx}", assv0.len());
    workbook.save(xlsx)?;
    Ok(())
}

pub fn excel_aoj_repo1(xlsx: &str, mxrw: Option<usize>) -> Result<(), Box<dyn Error>> {
    println!("cmd1");
    let aids = vec![
        "N1", "N2", "N3", "NE1", "NE2", "NE3", "C1", "C2", "C3", "S1", "S2", "S3",
    ];
    let mut aojm = HashMap::<String,String>::new();
    for aid in aids {
        let aojs0 = ProcEngine::aojs0(aid);
        for aoj in &aojs0 {
            let Some(ref aojcd) = aoj.code else {
                continue;
            };
            let Some(ref name) = aoj.sht_name else {
                continue;
            };
            aojm.insert(aojcd.to_string(), name.to_string());
        }
    }
    let mut workbook = Workbook::new();
    let mut flds = vec![VarType::FirSum, VarType::EirSum];
    let mut fld1 = FIR_FLDS.to_vec();
    flds.append(&mut fld1);
    let dnm = crate::dcl::get_dirnm();
    let Ok(buf) = std::fs::read(format!("{dnm}/000-aojrw.bin")) else {
        return Err("Not found".into());
    };
    // ==== read rw3 data
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    assv0.sort_by(|a, b| {
        let a0 = a.v[VarType::Uc1Rank.tousz()].v
            + a.v[VarType::Uc2Rank.tousz()].v
            + a.v[VarType::Uc3Rank.tousz()].v;
        let b0 = b.v[VarType::Uc1Rank.tousz()].v
            + b.v[VarType::Uc2Rank.tousz()].v
            + b.v[VarType::Uc3Rank.tousz()].v;
        a0.partial_cmp(&b0).unwrap()
    });

    let sht = workbook.add_worksheet();
    let _ = sht.set_name("ปริมาณงาน");
    sht.set_column_width(0, 5)?;
    sht.set_column_width(1, 20)?;
    let ttfm = Format::new().set_bold();
    let hdfm = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xC6EFCE));
    let mut rw = 1;
    let datanm = "ปริมาณงาน".to_string();
    let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
    rw += 2;
    let nos = "ลำดับ".to_string();
    let prv = "aoj".to_string();
    let _ = sht.write_with_format(rw, 0, nos, &hdfm);
    let _ = sht.write_with_format(rw, 1, prv, &hdfm);
    for (i, co) in DATA_FLDS.iter().enumerate() {
        let vas = format!("{co:?}");
        let _ = sht.write_with_format(rw, i as u16 + 2, vas, &hdfm);
    }
    let mut fmts = vec![];
    for dt in DATA_FLDS.iter() {
        let dtn = format!("{dt:?}");
        if dtn.ends_with("MWh") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else {
            fmts.push(Format::new().set_num_format("#,##0"));
        }
    }
    for (i, ass) in assv0.iter().enumerate() {
        if let Some(mxrw) = mxrw && i>=mxrw {
            break;
        }
        rw += 1;
        let mut co = 0;
        let no = i as i32 + 1;
        let _ = sht.write(rw, co, no);
        co += 1;
        let Some(aojnm) = aojm.get(&ass.aojcd) else {
            continue;
        };
        let _ = sht.write(rw, co, aojnm.clone());
        for (i, (co, fm)) in DATA_FLDS.iter().zip(fmts.iter()).enumerate() {
            let dt = ass.v[co.tousz()].v;
            let _ = sht.write_with_format(rw, i as u16 + 2, dt, fm)?;
        }
    }

    for f in flds.iter() {
        println!("{f:?}");
        let shtnm = format!("{f:?}");
        let sht = workbook.add_worksheet();
        let _ = sht.set_name(&shtnm);
        sht.set_column_width(0, 5)?;
        sht.set_column_width(1, 20)?;
        let ttfm = Format::new().set_bold();
        let defm = Format::new().set_num_format("#,##0");
        let hdfm = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xC6EFCE));
        for i in 0..15 {
            sht.set_column_width(2 + i, 15)?;
        }
        let mut rw = 1;
        let datanm = format!("ชื่อข้อมูล: {f:?}");
        let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
        rw += 2;
        let nos = "ลำดับ".to_string();
        let prv = "aoj".to_string();
        let _ = sht.write_with_format(rw, 0, nos, &hdfm);
        let _ = sht.write_with_format(rw, 1, prv, &hdfm);
        for i in 0..15 {
            let vas = format!("ปี{}", i + 2026);
            let _ = sht.write_with_format(rw, i + 2, vas, &hdfm);
        }

        for (i, ass) in assv0.iter().enumerate() {
            if let Some(mxrw) = mxrw && i>=mxrw {
                break;
            }
            rw += 1;
            let mut co = 0;
            let no = i as i32 + 1;
            let _ = sht.write(rw, co, no);
            co += 1;
            let Some(aojnm) = aojm.get(&ass.aojcd) else { continue; };
            let _ = sht.write(rw, co, aojnm.clone());
            for v in ass.vy[f.tousz()].iter() {
                co += 1;
                let _ = sht.write_with_format(rw, co, *v, &defm)?;
            }
        }
    }

    println!("SAVE {} in {xlsx}", assv0.len());
    workbook.save(xlsx)?;
    Ok(())
}
use std::collections::HashMap;
use crate::dcl::ProcEngine;

pub fn chk_aoj1() -> Result<(), Box<dyn Error>> {
    let aids = vec![
        "N1", "N2", "N3", "NE1", "NE2", "NE3", "C1", "C2", "C3", "S1", "S2", "S3",
    ];
    let mut aojm = HashMap::<String,String>::new();
    for aid in aids {
        let aojs0 = ProcEngine::aojs0(aid);
        for aoj in &aojs0 {
            let Some(ref aojcd) = aoj.code else {
                continue;
            };
            let Some(ref name) = aoj.sht_name else {
                continue;
            };
            aojm.insert(aojcd.to_string(), name.to_string());
        }
    }
    println!("{aojm:?}");
    Ok(())
}

/*
pub fn excel_prv_repo1(xlsx: &str, mxrw: Option<usize>) -> Result<(), Box<dyn Error>> {
    println!("prv {mxrw:?}");
    let mut workbook = Workbook::new();
    let mut flds = vec![VarType::FirSum, VarType::EirSum];
    let mut fld1 = FIR_FLDS.to_vec();
    flds.append(&mut fld1);
    let dnm = crate::dcl::get_dirnm();
    let Ok(buf) = std::fs::read(format!("{dnm}/000-pvrw.bin")) else {
        return Err("Not found".into());
    };
    // ==== read rw3 data
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    assv0.sort_by(|a, b| {
        let a0 = a.v[VarType::Uc1Rank.tousz()].v
            + a.v[VarType::Uc2Rank.tousz()].v
            + a.v[VarType::Uc3Rank.tousz()].v;
        let b0 = b.v[VarType::Uc1Rank.tousz()].v
            + b.v[VarType::Uc2Rank.tousz()].v
            + b.v[VarType::Uc3Rank.tousz()].v;
        a0.partial_cmp(&b0).unwrap()
    });

    let sht = workbook.add_worksheet();
    let _ = sht.set_name("ปริมาณงาน");
    sht.set_column_width(0, 5)?;
    sht.set_column_width(1, 20)?;
    let ttfm = Format::new().set_bold();
    let hdfm = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xC6EFCE));
    let mut rw = 1;
    let datanm = "ปริมาณงาน".to_string();
    let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
    rw += 2;
    let nos = "ลำดับ".to_string();
    let prv = "จังหวัด".to_string();
    let _ = sht.write_with_format(rw, 0, nos, &hdfm);
    let _ = sht.write_with_format(rw, 1, prv, &hdfm);
    for (i, co) in DATA_FLDS.iter().enumerate() {
        let vas = format!("{co:?}");
        let _ = sht.write_with_format(rw, i as u16 + 2, vas, &hdfm);
    }
    let mut fmts = vec![];
    for dt in DATA_FLDS.iter() {
        let dtn = format!("{dt:?}");
        if dtn.ends_with("MWh") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else {
            fmts.push(Format::new().set_num_format("#,##0"));
        }
    }
    for (i, ass) in assv0.iter().enumerate() {
        if let Some(mxrw) = mxrw && i>=mxrw {
            break;
        }
        rw += 1;
        let mut co = 0;
        let no = i as i32 + 1;
        let _ = sht.write(rw, co, no);
        co += 1;
        let _ = sht.write(rw, co, ass.pvid.clone());
        for (i, (co, fm)) in DATA_FLDS.iter().zip(fmts.iter()).enumerate() {
            let dt = ass.v[co.tousz()].v;
            let _ = sht.write_with_format(rw, i as u16 + 2, dt, fm)?;
        }
    }

    for f in flds.iter() {
        println!("{f:?}");
        let shtnm = format!("{f:?}");
        let sht = workbook.add_worksheet();
        let _ = sht.set_name(&shtnm);
        sht.set_column_width(0, 5)?;
        sht.set_column_width(1, 20)?;
        let ttfm = Format::new().set_bold();
        let defm = Format::new().set_num_format("#,##0");
        let hdfm = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xC6EFCE));
        for i in 0..15 {
            sht.set_column_width(2 + i, 15)?;
        }
        let mut rw = 1;
        let datanm = format!("ชื่อข้อมูล: {f:?}");
        let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
        rw += 2;
        let nos = "ลำดับ".to_string();
        let prv = "จังหวัด".to_string();
        let _ = sht.write_with_format(rw, 0, nos, &hdfm);
        let _ = sht.write_with_format(rw, 1, prv, &hdfm);
        for i in 0..15 {
            let vas = format!("ปี{}", i + 2026);
            let _ = sht.write_with_format(rw, i + 2, vas, &hdfm);
        }

        for (i, ass) in assv0.iter().enumerate() {
            if let Some(mxrw) = mxrw && i>=mxrw {
                break;
            }
            rw += 1;
            let mut co = 0;
            let no = i as i32 + 1;
            let _ = sht.write(rw, co, no);
            co += 1;
            let _ = sht.write(rw, co, ass.pvid.clone());
            for v in ass.vy[f.tousz()].iter() {
                co += 1;
                let _ = sht.write_with_format(rw, co, *v, &defm)?;
            }
        }
    }

    println!("SAVE {} in {xlsx}", assv0.len());
    workbook.save(xlsx)?;
    Ok(())
}
*/

use phf_macros::phf_map;

pub static PROV_SET1: phf::Map<&'static str, usize> = phf_map! {
"ชลบุรี" => 1,
"ปทุมธานี" => 2,
"นครราชสีมา" => 3,
"ระยอง" => 4,
"สมุทรสาคร" => 5,
"เชียงใหม่" => 6,
"นครปฐม" => 7,
"สระบุรี" => 8,
"พระนครศรีอยุธยา" => 9,
"สุพรรณบุรี" => 10,
"สุราษฎร์ธานี" => 11,
"ฉะเชิงเทรา" => 12,
"ราชบุรี" => 13,
"นครศรีธรรมราช" => 14,
"กาญจนบุรี" => 15,
"สงขลา" => 16,
"จันทบุรี" => 17,
"ขอนแก่น" => 18,
"นครสวรรค์" => 19,
"ภูเก็ต" => 20,
"พิษณุโลก" => 21,
"ชุมพร" => 22,
"ปราจีนบุรี" => 23,
"อุดรธานี" => 24,
"ลพบุรี" => 25,
};

pub fn excel_prv_repo2(xlsx: &str) -> Result<(), Box<dyn Error>> {
    let mut workbook = Workbook::new();
    let mut flds = vec![VarType::FirSum, VarType::EirSum];
    let mut fld1 = FIR_FLDS.to_vec();
    flds.append(&mut fld1);
    let dnm = crate::dcl::get_dirnm();
    let Ok(buf) = std::fs::read(format!("{dnm}/000-pvrw.bin")) else {
        return Err("Not found".into());
    };
    // ==== read rw3 data
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    assv0.sort_by(|a, b| {
        let a0 = a.v[VarType::Uc1Rank.tousz()].v
            + a.v[VarType::Uc2Rank.tousz()].v
            + a.v[VarType::Uc3Rank.tousz()].v;
        let b0 = b.v[VarType::Uc1Rank.tousz()].v
            + b.v[VarType::Uc2Rank.tousz()].v
            + b.v[VarType::Uc3Rank.tousz()].v;
        a0.partial_cmp(&b0).unwrap()
    });

    let sht = workbook.add_worksheet();
    let _ = sht.set_name("ปริมาณงาน");
    sht.set_column_width(0, 5)?;
    sht.set_column_width(1, 20)?;
    let ttfm = Format::new().set_bold();
    let hdfm = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xC6EFCE));
    let mut rw = 1;
    let datanm = "ปริมาณงาน".to_string();
    let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
    rw += 2;
    let nos = "ลำดับ".to_string();
    let prv = "จังหวัด".to_string();
    let _ = sht.write_with_format(rw, 0, nos, &hdfm);
    let _ = sht.write_with_format(rw, 1, prv, &hdfm);
    for (i, co) in DATA_FLDS.iter().enumerate() {
        let vas = format!("{co:?}");
        let _ = sht.write_with_format(rw, i as u16 + 2, vas, &hdfm);
    }
    let mut fmts = vec![];
    for dt in DATA_FLDS.iter() {
        let dtn = format!("{dt:?}");
        if dtn.ends_with("MWh") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else {
            fmts.push(Format::new().set_num_format("#,##0"));
        }
    }
    let mut no = 0;
    for (_i, ass) in assv0.iter().enumerate() {
        if PROV_SET1.get(&ass.pvid).is_some() { } else { continue; }
        rw += 1;
        let mut co = 0;
        no += 1;
        let _ = sht.write(rw, co, no);
        co += 1;
        let _ = sht.write(rw, co, ass.pvid.clone());
        for (i, (co, fm)) in DATA_FLDS.iter().zip(fmts.iter()).enumerate() {
            let dt = ass.v[co.tousz()].v;
            let _ = sht.write_with_format(rw, i as u16 + 2, dt, fm)?;
        }
    }
    /*
    rw += 1;
    for (i, _) in DATA_FLDS.iter().enumerate() {
        let _ = sht.write_formula(rw, i as u16 + 2, Formula::new("=SUM(C5:C29)"))?;
    }
    */

    for f in flds.iter() {
        println!("{f:?}");
        let shtnm = format!("{f:?}");
        let sht = workbook.add_worksheet();
        let _ = sht.set_name(&shtnm);
        sht.set_column_width(0, 5)?;
        sht.set_column_width(1, 20)?;
        let ttfm = Format::new().set_bold();
        let defm = Format::new().set_num_format("#,##0");
        let hdfm = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xC6EFCE));
        for i in 0..15 {
            sht.set_column_width(2 + i, 15)?;
        }
        let mut rw = 1;
        let datanm = format!("ชื่อข้อมูล: {f:?}");
        let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
        rw += 2;
        let nos = "ลำดับ".to_string();
        let prv = "จังหวัด".to_string();
        let _ = sht.write_with_format(rw, 0, nos, &hdfm);
        let _ = sht.write_with_format(rw, 1, prv, &hdfm);
        for i in 0..15 {
            let vas = format!("ปี{}", i + 2026);
            let _ = sht.write_with_format(rw, i + 2, vas, &hdfm);
        }

    let mut no = 0;
        for (_i, ass) in assv0.iter().enumerate() {
            if PROV_SET1.get(&ass.pvid).is_some() { } else { continue; }
            rw += 1;
            let mut co = 0;
            no += 1;
            let _ = sht.write(rw, co, no);
            co += 1;
            let _ = sht.write(rw, co, ass.pvid.clone());
            for v in ass.vy[f.tousz()].iter() {
                co += 1;
                let _ = sht.write_with_format(rw, co, *v, &defm)?;
            }
        }
    }

    println!("SAVE {} in {xlsx}", assv0.len());
    workbook.save(xlsx)?;
    Ok(())
}


