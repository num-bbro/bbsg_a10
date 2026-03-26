use crate::dcl::PeaAssVar;
//use crate::dcl::VarType;
//use crate::dcl::VarType;
//use crate::stg3::AojInfo;
//use crate::utl2::add_2_sum;
use crate::utl2::ass_reorder;
//use crate::utl2::fld_2_var;
use crate::utl2::tab_row_popu;
//use crate::utl2::val_2_form;
use crate::utl2::ScriptParam;
//use std::collections::HashMap;
use crate::dcl::VarType;
use crate::dcl::FIR_FLDS;
use rust_xlsxwriter::*;
use std::error::Error;

pub fn excel_cmd1() -> Result<(), Box<dyn Error>> {
    println!("cmd1");
    let mut workbook = Workbook::new();
    let mut flds = vec![VarType::FirSum];
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
    for f in flds.iter() {
        println!("{f:?}");
        let shtnm = format!("{f:?}");
        let sht = workbook.add_worksheet();
        let _ = sht.set_name(shtnm);
        sht.set_column_width(0, 5)?;
        sht.set_column_width(1, 20)?;
        let defm = Format::new().set_num_format("#,##0.00");
        let hdfm = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xC6EFCE));
        for i in 0..15 {
            sht.set_column_width(2 + i, 15)?;
        }
        let mut rw = 3;
        let nos = "ลำดับ".to_string();
        let prv = "จังหวัด".to_string();
        let _ = sht.write_with_format(rw, 0, nos, &hdfm);
        let _ = sht.write_with_format(rw, 1, prv, &hdfm);
        for i in 0..15 {
            let vas = format!("ปี{}", i + 2026);
            let _ = sht.write_with_format(rw, i + 2, vas, &hdfm);
        }

        for (i, ass) in assv0.iter().enumerate() {
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
    println!("SAVE {}", assv0.len());
    workbook.save("cmd1.xlsx")?;
    Ok(())
}

use crate::utl::trf_kva_2_kw;

pub fn excel_cmd2() -> Result<(), Box<dyn Error>> {
    println!("cmd1");
    let mut workbook = Workbook::new();
    let mut flds = vec![VarType::FirSum];
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
    for f in flds.iter() {
        println!("{f:?}");
        let shtnm = format!("{f:?}");
        let sht = workbook.add_worksheet();
        let _ = sht.set_name(shtnm);
        sht.set_column_width(0, 5)?;
        sht.set_column_width(1, 20)?;
        let defm = Format::new().set_num_format("#,##0.00");
        let hdfm = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xC6EFCE));
        for i in 0..15 {
            sht.set_column_width(2 + i, 15)?;
        }
        let mut rw = 3;
        let nos = "ลำดับ".to_string();
        let sub = "สถานีไฟฟ้า".to_string();
        let max = "กำลังไฟฟ้าสูงสุด".to_string();
        let cap = "กำลังหม้อแปลง".to_string();
        let per = "UF".to_string();
        let _ = sht.write_with_format(rw, 0, nos, &hdfm);
        let _ = sht.write_with_format(rw, 1, sub, &hdfm);
        let _ = sht.write_with_format(rw, 2, max, &hdfm);
        let _ = sht.write_with_format(rw, 3, cap, &hdfm);
        let _ = sht.write_with_format(rw, 4, per, &hdfm);
        /*
        for i in 0..15 {
            let vas = format!("ปี{}", i + 2026);
            let _ = sht.write_with_format(rw, i + 2, vas, &hdfm);
        }
        */

        for (i, ass) in assv0.iter().enumerate() {
            rw += 1;
            let no = i as i32 + 1;
            let _ = sht.write(rw, 0, no);
            let _ = sht.write(rw, 1, ass.sbid.clone());
            let max = ass.v[VarType::MaxPosPowSub.tousz()].v;
            let _ = sht.write_with_format(rw, 2, max, &defm);
            let cap = trf_kva_2_kw(ass.v[VarType::SubPowCap.tousz()].v);
            let _ = sht.write_with_format(rw, 3, cap, &defm);
            let per = max / cap * 100.0;
            let _ = sht.write_with_format(rw, 4, per, &defm);
            /*
                sbas.v[VarType::MvPowSatTr as usize].v = sbas.v[VarType::MaxPosPowSub as usize].v
                    / z2o(sbas.v[VarType::SubPowCap as usize].v);
            for v in ass.vy[f.tousz()].iter() {
                co += 1;
                let _ = sht.write_with_format(rw, co, *v, &defm)?;
            }
            */
        }
    }
    println!("SAVE {}", assv0.len());
    workbook.save("cmd2.xlsx")?;
    Ok(())
}

pub fn ass_var_aoj(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    let dum = vec![vec!["1.1".to_string()]];
    //println!("ass var prv sc:{sc:?}");
    let dnm = crate::dcl::get_dirnm();
    let buf = std::fs::read(format!("{dnm}/000-aojrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    /*
    let buf = std::fs::read(format!("{DNM}/000-aojm.bin")).unwrap();
    let Ok((aojm, _)): Result<(HashMap<String, AojInfo>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    */
    if !sc.pvid.is_empty() {
        assv0 = assv0
            .into_iter()
            .filter(|a| a.pvid == sc.pvid)
            .collect::<_>();
    }
    ass_reorder(&mut assv0, sc);
    /*
    println!("AOJ : {}", assv0.len());
    for a in assv0.iter() {
        let n = a.vy[VarType::NoHmChgEvTr.tousz()].len();
        println!("aoj {} - {n}", a.aoj);
    }
    */
    tab_row_popu(sc, &assv0, "A")

    /*
    assv0.sort_by(|a, b| {
        let a0 = a.v[VarType::Uc1Rank.tousz()].v
            + a.v[VarType::Uc2Rank.tousz()].v
            + a.v[VarType::Uc3Rank.tousz()].v;
        let b0 = b.v[VarType::Uc1Rank.tousz()].v
            + b.v[VarType::Uc2Rank.tousz()].v
            + b.v[VarType::Uc3Rank.tousz()].v;
        a0.partial_cmp(&b0).unwrap()
    });
    println!("ass: {}", assv0.len());
    let mut vv = Vec::<Vec<String>>::new();
    let mut vsm = Vec::<f32>::new();
    for (i, a) in assv0.iter().enumerate() {
        if sc.lmt > 0 && i >= sc.lmt {
            break;
        }
        let mut v = Vec::<String>::new();
        v.push((i + 1).to_string());
        let aojx = if let Some(ao) = aojm.get(&a.aoj) {
            ao.name.clone().unwrap_or(String::new()).to_string()
            //format!("{}-{}", a.aoj, nm)
        } else {
            a.aoj.clone()
        };
        let aojx = format!("{aojx} ({})", a.aoj);
        v.push(aojx);
        let mut vas = Vec::<f32>::new();
        for (fd, pn) in sc.fld.iter().zip(sc.pan.iter()).skip(2) {
            let i = fld_2_var(fd).tousz();
            let va = a.v[i].v;
            let vp = val_2_form(va, pn.as_str());
            if sc.sum {
                vas.push(va);
            }
            v.push(vp);
        }
        if sc.sum {
            vsm = add_2_sum(vsm, vas);
        }
        vv.push(v);
    }
    if sc.sum {
        let mut v = Vec::<String>::new();
        v.push("".to_string());
        v.push("".to_string());
        for (i, va) in vsm.iter().enumerate() {
            let pn = sc.pan[i + 2].clone();
            let mut vp = val_2_form(*va, pn.as_str());
            if pn == "P" {
                vp = "".to_string();
            }
            v.push(vp);
        }
        vv.push(v);
    }
    vv
        */
}

pub fn ass_var_aoj_tr(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    let dum = vec![vec!["1.1".to_string()]];
    if sc.aojcd.is_empty() {
        return dum;
    }
    let aojcd = sc.aojcd[0].to_string();
    println!("AOJCD = {:?}", aojcd);

    //println!("ass var prv sc:{sc:?}");
    //    let fnm = format!("{DNM}/AOJ-{k}-assrw.bin");
    let dnm = crate::dcl::get_dirnm();
    let buf = std::fs::read(format!("{dnm}/AOJ-{aojcd}-assrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    /*
    let buf = std::fs::read(format!("{DNM}/000-aojm.bin")).unwrap();
    let Ok((aojm, _)): Result<(HashMap<String, AojInfo>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    if !sc.pvid.is_empty() {
        assv0 = assv0
            .into_iter()
            .filter(|a| a.pvid == sc.pvid)
            .collect::<_>();
    }
    */
    ass_reorder(&mut assv0, sc);
    tab_row_popu(sc, &assv0, "T")

    /*
    assv0.sort_by(|a, b| {
        let a0 = a.v[VarType::Uc1Rank.tousz()].v
            + a.v[VarType::Uc2Rank.tousz()].v
            + a.v[VarType::Uc3Rank.tousz()].v;
        let b0 = b.v[VarType::Uc1Rank.tousz()].v
            + b.v[VarType::Uc2Rank.tousz()].v
            + b.v[VarType::Uc3Rank.tousz()].v;
        a0.partial_cmp(&b0).unwrap()
    });
    println!("ass: {}", assv0.len());
    let mut vv = Vec::<Vec<String>>::new();
    let mut vsm = Vec::<f32>::new();
    for (i, a) in assv0.iter().enumerate() {
        if sc.lmt > 0 && i >= sc.lmt {
            break;
        }
        //println!("   SBID: {}", a.sbid);
        if sc.sbid.len() == 3 && a.sbid != sc.sbid {
            continue;
        }
        let mut v = Vec::<String>::new();
        v.push((i + 1).to_string());
        v.push(a.peano.clone());
        let mut vas = Vec::<f32>::new();
        for (fd, pn) in sc.fld.iter().zip(sc.pan.iter()).skip(2) {
            let i = fld_2_var(fd).tousz();
            let va = a.v[i].v;
            let vp = val_2_form(va, pn.as_str());
            if sc.sum {
                vas.push(va);
            }
            v.push(vp);
        }
        if sc.sum {
            vsm = add_2_sum(vsm, vas);
        }
        vv.push(v);
    }
    if sc.sum {
        let mut v = Vec::<String>::new();
        v.push("".to_string());
        v.push("".to_string());
        for (i, va) in vsm.iter().enumerate() {
            let pn = sc.pan[i + 2].clone();
            let mut vp = val_2_form(*va, pn.as_str());
            if pn == "P" {
                vp = "".to_string();
            }
            v.push(vp);
        }
        vv.push(v);
    }
    vv
        */
}

use crate::dcl::ProcEngine;

pub fn check_aoj() -> Result<(), Box<dyn Error>> {
    let ars = [
        "N1", "N2", "N3", "C1", "C2", "C3", "NE1", "NE2", "NE3", "S1", "S2", "S3",
    ];
    for a in ars {
        println!("area {a}");
        let aojs0 = ProcEngine::aojs0(a);
        println!("   aojs:{}", aojs0.len());
    }
    Ok(())
}
