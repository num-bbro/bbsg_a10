use crate::asm::ASM;
use crate::dcl::PeaAssVar;
use crate::dcl::VarType;
use crate::dcl::FIR_FLDS;
use crate::p08::ld_sub_info;
use crate::utl4::NumValEnum;
use crate::utl6::ArchiInfo;
use crate::utl7::get_brn_map;
use chrono::Local;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use shapefile::dbase;
use dbase::yore::code_pages::CP874;
use crate::img::fda01::meter_pixel_to_zoom_lat_2;
use crate::img::fda01::MP_HH;
use crate::img::fda01::MP_MG;
use crate::img::fda01::MP_UPDW;
use crate::img::fda01::MP_WW;
use sglab02_lib::sg::mvline::utm_latlong;
use sglib04::aoj::zoom_to_meter_pixel_lat;
use crate::utl7::DbfVal;
//use dbase::CodePageMark::CP874;
use crate::utl7::db_rec;

use crate::dcl::Pan;

use crate::sty3::AssSumEnum;
use crate::sty3::AssSumEnum::*;
use crate::utl7::EconCalcInfo;

pub fn fld_name_2_format(dtn: &str) -> &'static str {
    if dtn.ends_with("MWh") {
        "#,##0.00"
    } else if dtn.ends_with("IRR") {
        "#,##0.00"
    } else if dtn.ends_with("Year") {
        "#,##0.00"
    } else if dtn.ends_with("Rate") {
        "#,##0.00"
    } else {
        "#,##0"
    }
}

//use phf_macros::phf_map;

//pub static FLD_TITLE: phf::Map<VarType, &'static str> = phf_map! {

pub fn fld_title(fld: VarType) -> String {
    match fld {
        VarType::NoMet1Ph => "มิเตอร์1ฟ".to_string(),
        VarType::NoMet3Ph => "มิเตอร์3ฟ".to_string(),
        VarType::NoPeaTr => "ม.หม้อแปลง".to_string(),
        VarType::BessMWh => "BESS".to_string(),
        VarType::TpoAdd => "TPO".to_string(),
        VarType::EcuAdd => "ECU".to_string(),
        VarType::EcoReturn => "รายได้สะสม".to_string(),
        VarType::EcoCapex => "CAPEX".to_string(),
        VarType::EcoOpex => "OPEX".to_string(),
        VarType::EcoInterest => "ดอกเบี้ยสะสม".to_string(),
        VarType::EcoCost => "ค่าใช้จ่ายรวม".to_string(),
        VarType::EcoNpv => "NPV".to_string(),
        VarType::EcoIRR => "IRR".to_string(),
        VarType::EcoBreakYear => "คืนทุน".to_string(),
        _ => format!("{fld:?}"),
    }
}

pub fn excel_repo1(arif: &ArchiInfo, sumtp: AssSumEnum, phs: i32) -> Result<(), Box<dyn Error>> {
    println!("PHASE {phs}");
    let dnm = arif.ass.t(ASM::OUTDIR);
    let xlsx = arif.ass.t(ASM::XLSX_OUT);
    let tmst = Local::now().format("D%Y%m%dT%H%M%S").to_string();
    let xlsx = xlsx.replace("#T1#", &tmst);
    //println!("outfile: {xlsx}");

    let maxrw = arif.ass.v(ASM::MAX_ROW_NO);
    let maxrw = maxrw as usize;
    //println!("MAX ROW: {maxrw}");
    let sbif = ld_sub_info();

    let flds = arif.ass.ve(ASM::DATA_FIELDS)?;
    //println!("DATA:");
    let mut datas = vec![];
    if let NumValEnum::Json(Value::Array(flds)) = flds {
        for x in flds.iter() {
            let Value::String(x) = x else {
                continue;
            };
            let a = VarType::from_str(x)?;
            //let s = x.to_string();
            //println!(" {a:?}");
            datas.push(a);
        }
    }

    let shts = arif.ass.ve(ASM::SHEET_FIELDS)?;
    //println!("SHEETS:");
    let mut sheets = vec![];
    if let NumValEnum::Json(Value::Array(shts)) = shts {
        for x in shts.iter() {
            let Value::String(x) = x else {
                continue;
            };
            let a = VarType::from_str(x)?;
            //println!(" {a:?}");
            sheets.push(a);
        }
    }

    //println!("PROVINCES:");
    let prvs: Vec<String> = if let Ok(prvs) = arif.ass.ve(ASM::PROVINCE_LIST) {
        let mut provs = vec![];
        if let NumValEnum::Json(Value::Array(prvs)) = prvs {
            for x in prvs.iter() {
                let Value::String(x) = x else {
                    continue;
                };
                let s = x.to_string();
                //println!(" {s}");
                provs.push(s);
            }
        }
        provs
    } else {
        vec![]
    };

    //println!("PROVINCE: {}", prvs.len());
    let fnm = match sumtp {
        SumPrvSub => format!("asssum-{:?}.bin", SumPrvSub),
        SumSub => format!("asssum-{:?}.bin", SumSub),
        SumPrvBrn => format!("asssum-{:?}.bin", SumPrvBrn),
        SumBrn => format!("asssum-{:?}.bin", SumBrn),
        SumPrvBrn1 => format!("asssum-{:?}.bin", SumPrvBrn1),
        SumBrn1 => format!("asssum-{:?}.bin", SumBrn1),
        SumPrvBrn2 => format!("asssum-{:?}.bin", SumPrvBrn2),
        SumBrn2 => format!("asssum-{:?}.bin", SumBrn2),
    };

    let hed = match sumtp {
        SumPrvSub | SumPrvBrn | SumPrvBrn1 | SumPrvBrn2 => "จังหวัด",
        SumSub => "สถานีไฟฟ้า",
        SumBrn | SumBrn1 | SumBrn2 => "กฟส",
    };

    let mut workbook = Workbook::new();
    let flds = sheets.clone();
    let Ok(buf) = std::fs::read(format!("{dnm}/{fnm}")) else {
        return Err(format!("Not found {dnm}/{fnm}").into());
    };
    let Ok((assv0, _)): Result<(Vec<(PeaAssVar, Vec<usize>)>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };

    let mut assv0 = assv0
        .iter()
        .map(|(a, _)| a.clone())
        .collect::<Vec<PeaAssVar>>();

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
    */

    let (brnv, brni) = get_brn_map()?;
    match sumtp {
        SumPrvBrn2 if phs == 2 => {
            let econs = if let AssSumEnum::SumPrvBrn2 = sumtp {
                println!("===== SUM =====");
                let fnm = format!("{dnm}/econ-calc-{:?}.bin", AssSumEnum::SumPrvBrn2);
                let Ok(buf) = std::fs::read(fnm) else {
                    return Err("File '{fnm}' not found'".into());
                };
                let Ok((econ, _)): Result<(Vec<EconCalcInfo>, usize), _> =
                    bincode::decode_from_slice(&buf[..], bincode::config::standard())
                else {
                    return Err("File '{fnm}' cannot be decoded'".into());
                };
                econ
            } else {
                return Err("File '{fnm}' cannot be decoded'".into());
            };
            println!("ECON: {}", econs.len());
            //let mut econs_i = vec![];
            let mut econs_i = econs
                .iter()
                .enumerate()
                .map(|(i, _)| i)
                .collect::<Vec<usize>>();
            econs_i.sort_by(|a, b| econs[*b].npv.partial_cmp(&econs[*a].npv).unwrap());
            /*
            let econs_i = econs_i
                .iter()
                .filter(|a| econs[**a].irr > 0.3)
                .map(|a| *a)
                .collect::<Vec<_>>();
            let econs_i = econs_i.iter().take(25).map(|a| *a).collect::<Vec<_>>();
            */
            let mut pv_assi = HashMap::<String, usize>::new();
            for (i, ass) in assv0.iter().enumerate() {
                pv_assi.insert(ass.pvid.clone(), i);
            }
            let mut assv1 = Vec::<PeaAssVar>::new();
            println!("PROVINCE AFTER CALC");

            use crate::dcl::Pan;

            for (ni, ii) in econs_i.iter().enumerate() {
                let pvid = econs[*ii].pvid.clone();
                println!("{ni}. {ii}. {pvid} npv:{}", econs[*ii].npv.pan(0));
                if let Some(ui) = pv_assi.get(&pvid) {
                    let mut ass = assv0[*ui].clone();
                    ass.v[VarType::EcoReturn as usize].v = econs[*ii].iret;
                    ass.v[VarType::EcoCapex as usize].v = econs[*ii].capex;
                    ass.v[VarType::EcoOpex as usize].v = econs[*ii].opex;
                    ass.v[VarType::EcoInterest as usize].v = econs[*ii].intrs;
                    ass.v[VarType::EcoCost as usize].v = econs[*ii].cost;
                    ass.v[VarType::EcoNpv as usize].v = econs[*ii].npv;
                    ass.v[VarType::EcoIRR as usize].v = econs[*ii].irr;
                    ass.v[VarType::EcoBreakYear as usize].v = econs[*ii].brkyr;
                    assv1.push(ass);
                }
            }
            assv0 = assv1;
        }
        SumPrvSub | SumPrvBrn | SumPrvBrn1 | SumPrvBrn2 => {
            if !prvs.is_empty() {
                let mut pv_ix = HashMap::<String, usize>::new();
                for (i, ass) in assv0.iter().enumerate() {
                    pv_ix.insert(ass.pvid.to_string(), i);
                }
                let mut assv1 = Vec::<PeaAssVar>::new();
                for pv in prvs.iter() {
                    if let Some(iu) = pv_ix.get(pv) {
                        assv1.push(assv0[*iu].clone());
                    } else {
                        let mut ass = PeaAssVar::from(0u64);
                        ass.pvid = pv.to_string();
                        assv1.push(ass);
                    }
                }
                assv0 = assv1;
            } else {
                assv0.sort_by(|a, b| a.pvid.cmp(&b.pvid));
            }
        }
        SumSub => {
            assv0.sort_by(|a, b| a.pvid.cmp(&b.pvid));
        }
        SumBrn | SumBrn1 | SumBrn2 => {
            if !brnv.is_empty() {
                let mut brn_ix = HashMap::<String, usize>::new();
                for (i, ass) in assv0.iter().enumerate() {
                    brn_ix.insert(ass.aojcd.to_string(), i);
                }
                let mut assv1 = Vec::<PeaAssVar>::new();
                for brn in brnv.iter() {
                    if let Some(iu) = brn_ix.get(&brn.code) {
                        assv1.push(assv0[*iu].clone());
                    }
                }
                assv0 = assv1;
            }
        }
    }
    if maxrw > 0 {
        let mut assv1 = Vec::<PeaAssVar>::new();
        for i in 0..maxrw {
            assv1.push(assv0[i].clone());
        }
        assv0 = assv1;
    }
    //let brnif = get_brn_info()?;

    //================ SHEET #1
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
    //let prv = "จังหวัด".to_string();
    let prv = hed.to_string();
    let _ = sht.write_with_format(rw, 0, nos, &hdfm);
    let _ = sht.write_with_format(rw, 1, prv, &hdfm);
    //for (i, co) in DATA_FLDS.iter().enumerate() {
    for (i, co) in datas.iter().enumerate() {
        sht.set_column_width(i as u16 + 2, 15)?;
        let vas = fld_title(co.clone());
        //let vas = format!("{co:?}");
        let _ = sht.write_with_format(rw, i as u16 + 2, vas, &hdfm);
    }
    let mut fmts = vec![];
    for dt in datas.iter() {
        let dtn = format!("{dt:?}");
        fmts.push(Format::new().set_num_format(fld_name_2_format(&dtn)));
        /*
        if dtn.ends_with("MWh") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else if dtn.ends_with("Rate") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else {
            fmts.push(Format::new().set_num_format("#,##0"));
        }
        */
    }
    use crate::dcl::Geo;
    for (i, ass) in assv0.iter().enumerate() {
        rw += 1;
        let mut co = 0;
        let no = i as i32 + 1;
        let _ = sht.write(rw, co, no);
        co += 1;
        let nm = match sumtp {
            SumPrvSub | SumPrvBrn | SumPrvBrn1 | SumPrvBrn2 => ass.pvid.clone(),
            SumSub => {
                let map = if let Some(sbinf) = sbif.get(&ass.sbid) {
                    sbinf.name.clone()
                } else {
                    "".to_string()
                };
                let (x, y) = ass.n1d.n1d_2_latlon();
                format!(
                    "'จ.{}'-'{}'-'ส.{}' L:[{},{}]",
                    ass.pvid, ass.sbid, map, x, y
                )
            }
            SumBrn | SumBrn1 | SumBrn2 => {
                let (nm, iv, pv, sz) = if let Some(i) = brni.get(&ass.aojcd) {
                    let name = if brnv[*i].is_prv {
                        format!("กฟจ.{}", brnv[*i].name)
                    } else {
                        format!("กฟส.{}", brnv[*i].name)
                    };
                    let stock = if let Some(jv) = brnv[*i].i_stock {
                        brnv[jv].name.clone()
                    } else {
                        "".to_string()
                    };
                    let prov = if let Some(jv) = brnv[*i].i_prov {
                        brnv[jv].name.clone()
                    } else {
                        "".to_string()
                    };
                    (name, stock, prov, brnv[*i].size.to_string())
                } else {
                    (
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                    )
                };
                format!("'{}'-'{nm}' คลัง:{iv} จ.{pv} size:'{sz}'", ass.aojcd)
            }
        };
        let _ = sht.write(rw, co, nm);
        //let _ = sht.write(rw, co, ass.pvid.clone());
        //for (i, (co, fm)) in DATA_FLDS.iter().zip(fmts.iter()).enumerate() {
        for (i, (co, fm)) in datas.iter().zip(fmts.iter()).enumerate() {
            let dt = ass.v[co.tousz()].v;
            let _ = sht.write_with_format(rw, i as u16 + 2, dt, fm)?;
        }
    }

    //================ SHEET #2 BEGIN =================
    let sht = workbook.add_worksheet();
    let _ = sht.set_name("รายได้");
    sht.set_column_width(0, 5)?;
    sht.set_column_width(1, 20)?;
    let ttfm = Format::new().set_bold();
    let hdfm = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xC6EFCE));
    let mut rw = 1;
    let datanm = "รายได้".to_string();
    let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
    rw += 2;
    let nos = "ลำดับ".to_string();
    //let prv = "จังหวัด".to_string();
    let prv = hed.to_string();
    let _ = sht.write_with_format(rw, 0, nos, &hdfm);
    let _ = sht.write_with_format(rw, 1, prv, &hdfm);

    let mut datas = vec![VarType::FirSum];
    for vt in FIR_FLDS.iter() {
        datas.push(vt.clone());
    }
    for (i, co) in datas.iter().enumerate() {
        sht.set_column_width(i as u16 + 2, 15)?;
        let vas = format!("{co:?}");
        let _ = sht.write_with_format(rw, i as u16 + 2, vas, &hdfm);
    }
    let mut fmts = vec![];
    //for dt in DATA_FLDS.iter() {
    for dt in datas.iter() {
        let dtn = format!("{dt:?}");
        fmts.push(Format::new().set_num_format(fld_name_2_format(&dtn)));
        /*
        if dtn.ends_with("MWh") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else {
            fmts.push(Format::new().set_num_format("#,##0"));
        }
        */
    }
    for (i, ass) in assv0.iter().enumerate() {
        rw += 1;
        let mut co = 0;
        let no = i as i32 + 1;
        let _ = sht.write(rw, co, no);
        co += 1;
        let nm = match sumtp {
            SumPrvSub | SumPrvBrn | SumPrvBrn1 | SumPrvBrn2 => ass.pvid.clone(),
            SumSub => {
                let map = if let Some(sbinf) = sbif.get(&ass.sbid) {
                    sbinf.name.clone()
                } else {
                    "".to_string()
                };
                let (x, y) = ass.n1d.n1d_2_latlon();
                format!(
                    "'จ.{}'-'{}'-'ส.{}' L:[{},{}]",
                    ass.pvid, ass.sbid, map, x, y
                )
            }
            SumBrn | SumBrn1 | SumBrn2 => {
                let (nm, iv, pv, sz) = if let Some(i) = brni.get(&ass.aojcd) {
                    let name = if brnv[*i].is_prv {
                        format!("กฟจ.{}", brnv[*i].name)
                    } else {
                        format!("กฟส.{}", brnv[*i].name)
                    };
                    let stock = if let Some(jv) = brnv[*i].i_stock {
                        brnv[jv].name.clone()
                    } else {
                        "".to_string()
                    };
                    let prov = if let Some(jv) = brnv[*i].i_prov {
                        brnv[jv].name.clone()
                    } else {
                        "".to_string()
                    };
                    (name, stock, prov, brnv[*i].size.to_string())
                } else {
                    (
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                    )
                };
                format!("'{}'-'{nm}' คลัง:{iv} จ.{pv} size:'{sz}'", ass.aojcd)
            }
        };
        let _ = sht.write(rw, co, nm);
        //let _ = sht.write(rw, co, ass.pvid.clone());
        //for (i, (co, fm)) in DATA_FLDS.iter().zip(fmts.iter()).enumerate() {
        for (i, (co, fm)) in datas.iter().zip(fmts.iter()).enumerate() {
            let dt = ass.v[co.tousz()].v;
            let _ = sht.write_with_format(rw, i as u16 + 2, dt, fm)?;
        }
    }
    //================ SHEET #2 END =================

    //================ SHEET #3 BEGIN =================
    let sht = workbook.add_worksheet();
    let _ = sht.set_name("ค่าใช้จ่าย");
    sht.set_column_width(0, 5)?;
    sht.set_column_width(1, 20)?;
    let ttfm = Format::new().set_bold();
    let hdfm = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xC6EFCE));
    let mut rw = 1;
    let datanm = "ค่าใช้จ่าย".to_string();
    let _ = sht.write_with_format(rw, 0, datanm, &ttfm);
    rw += 2;
    let nos = "ลำดับ".to_string();
    //let prv = "จังหวัด".to_string();
    let prv = hed.to_string();
    let _ = sht.write_with_format(rw, 0, nos, &hdfm);
    let _ = sht.write_with_format(rw, 1, prv, &hdfm);

    use crate::dcl::CAPEX_FLDS;
    use crate::dcl::OPEX_FLDS;

    let mut datas = vec![
        VarType::FirCstRate,
        VarType::CstCapEx,
        VarType::CstOpEx,
        VarType::CstCapOpEx,
    ];
    for vt in CAPEX_FLDS.iter() {
        datas.push(vt.clone());
    }
    for vt in OPEX_FLDS.iter() {
        datas.push(vt.clone());
    }
    for (i, co) in datas.iter().enumerate() {
        sht.set_column_width(i as u16 + 2, 15)?;
        let vas = format!("{co:?}");
        let _ = sht.write_with_format(rw, i as u16 + 2, vas, &hdfm);
    }
    let mut fmts = vec![];
    //for dt in DATA_FLDS.iter() {
    for dt in datas.iter() {
        let dtn = format!("{dt:?}");
        fmts.push(Format::new().set_num_format(fld_name_2_format(&dtn)));
        /*
        if dtn.ends_with("MWh") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else {
            fmts.push(Format::new().set_num_format("#,##0"));
        }
        */
    }
    for (i, ass) in assv0.iter().enumerate() {
        rw += 1;
        let mut co = 0;
        let no = i as i32 + 1;
        let _ = sht.write(rw, co, no);
        co += 1;
        let nm = match sumtp {
            SumPrvSub | SumPrvBrn | SumPrvBrn1 | SumPrvBrn2 => ass.pvid.clone(),
            SumSub => {
                let map = if let Some(sbinf) = sbif.get(&ass.sbid) {
                    sbinf.name.clone()
                } else {
                    "".to_string()
                };
                let (x, y) = ass.n1d.n1d_2_latlon();
                format!(
                    "'จ.{}'-'{}'-'ส.{}' L:[{},{}]",
                    ass.pvid, ass.sbid, map, x, y
                )
            }
            SumBrn | SumBrn1 | SumBrn2 => {
                let (nm, iv, pv, sz) = if let Some(i) = brni.get(&ass.aojcd) {
                    let name = if brnv[*i].is_prv {
                        format!("กฟจ.{}", brnv[*i].name)
                    } else {
                        format!("กฟส.{}", brnv[*i].name)
                    };
                    let stock = if let Some(jv) = brnv[*i].i_stock {
                        brnv[jv].name.clone()
                    } else {
                        "".to_string()
                    };
                    let prov = if let Some(jv) = brnv[*i].i_prov {
                        brnv[jv].name.clone()
                    } else {
                        "".to_string()
                    };
                    (name, stock, prov, brnv[*i].size.to_string())
                } else {
                    (
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                    )
                };
                format!("'{}'-'{nm}' คลัง:{iv} จ.{pv} size:'{sz}'", ass.aojcd)
            }
        };
        let _ = sht.write(rw, co, nm);
        //let _ = sht.write(rw, co, ass.pvid.clone());
        //for (i, (co, fm)) in DATA_FLDS.iter().zip(fmts.iter()).enumerate() {
        for (i, (co, fm)) in datas.iter().zip(fmts.iter()).enumerate() {
            let dt = ass.v[co.tousz()].v;
            let _ = sht.write_with_format(rw, i as u16 + 2, dt, fm)?;
        }
    }
    //================ SHEET #3 END =================

    //=============== SHEET #4 ================
    for f in flds.iter() {
        //println!("{f:?}");
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
        //let prv = "จังหวัด".to_string();
        let prv = hed.to_string();
        let _ = sht.write_with_format(rw, 0, nos, &hdfm);
        let _ = sht.write_with_format(rw, 1, prv, &hdfm);
        for i in 0..15 {
            let vas = format!("ปี{}", i + 2026);
            sht.write_with_format(rw, i + 2, vas, &hdfm)?;
        }

        for (i, ass) in assv0.iter().enumerate() {
            /*
            if let Some(mxrw) = mxrw && i>=mxrw {
                break;
            }
            */
            rw += 1;
            let mut co = 0;
            let no = i as i32 + 1;
            let _ = sht.write(rw, co, no);
            co += 1;
            let nm = match sumtp {
                SumPrvSub | SumPrvBrn | SumPrvBrn1 | SumPrvBrn2 => ass.pvid.clone(),
                SumSub => {
                    let map = if let Some(sbinf) = sbif.get(&ass.sbid) {
                        sbinf.name.clone()
                    } else {
                        "".to_string()
                    };
                    let (x, y) = ass.n1d.n1d_2_latlon();
                    format!(
                        "'จ.{}'-'{}'-'ส.{}' L:[{},{}]",
                        ass.pvid, ass.sbid, map, x, y
                    )
                }
                SumBrn | SumBrn1 | SumBrn2 => {
                    let (nm, iv, pv, sz) = if let Some(i) = brni.get(&ass.aojcd) {
                        let name = if brnv[*i].is_prv {
                            format!("กฟจ.{}", brnv[*i].name)
                        } else {
                            format!("กฟส.{}", brnv[*i].name)
                        };
                        let stock = if let Some(jv) = brnv[*i].i_stock {
                            brnv[jv].name.clone()
                        } else {
                            "".to_string()
                        };
                        let prov = if let Some(jv) = brnv[*i].i_prov {
                            brnv[jv].name.clone()
                        } else {
                            "".to_string()
                        };
                        (name, stock, prov, brnv[*i].size.to_string())
                    } else {
                        (
                            "".to_string(),
                            "".to_string(),
                            "".to_string(),
                            "".to_string(),
                        )
                    };
                    format!("'{}'-'{nm}' คลัง:{iv} จ.{pv} size:'{sz}'", ass.aojcd)
                }
            };
            sht.write(rw, co, nm)?;
            //let _ = sht.write(rw, co, ass.pvid.clone());
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

use crate::utl6::archi_analyze;
use crate::utl6::archi_extract0;
use crate::utl6::archi_xml_read0;
use crate::utl6::get_assum_in_view;
use crate::utl6::ARCHI_INPUT;
use crate::utl8::ASM::OUTDIR;

pub fn sum_proc_e(_coreno: usize, vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let dnm = arif.ass.t(OUTDIR);
    let fnm = format!("{dnm}/all-rw4.bin");
    println!("..sg_proc5 fnm:{fnm}");
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("NO {fnm} file:").into());
    };
    println!("READ BUF {}", buf.len());
    let Ok((assv, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Failed to decode rw3:".into());
    };
    println!("assv: {}", assv.len());
    Ok(())
}

const SUB_RPF: [&str; 35] = [
    "ARA", "ATB", "BEA", "BNN", "BOP", "BPB", "BPI", "BSG", "BSP", "CHM", "CHN", "DAA", "DMA",
    "KBA", "KKU", "KON", "KPA", "LLA", "NOA", "NQA", "NSI", "PIH", "PKT", "PTN", "PTU", "PVA",
    "SIA", "SKW", "SYA", "TSE", "TYI", "WAM", "WBA", "WIR", "WYA",
];

const SUB_OVLD: [&str; 22] = [
    "BBB", "BKO", "BOA", "BSM", "CRB", "EKA", "KCD", "KMB", "KRN", "KUP", "LOA", "MKA", "NPV",
    "NRD", "NSA", "PLX", "PPA", "PTP", "PYB", "RAD", "SCA", "SKP",
];

const SUB_RPF_OVLD: [&str; 52] = [
    "BSR", "BBA", "BUY", "DEA", "DKA", "DOA", "DTA", "HCA", "HTL", "KAA", "KGA", "KKN", "KPB",
    "KSR", "KSW", "KTB", "KUA", "KUR", "MDB", "NAR", "NBL", "NGA", "NOB", "NOK", "NRE", "PBB",
    "PCB", "PIA", "PST", "QKA", "QLA", "REB", "RKA", "RLA", "SEK", "SLL", "SMP", "SOB", "SPO",
    "SRB", "STT", "TCA", "TEA", "THA", "TPH", "TTP", "UBB", "WEA", "WIA", "WNA", "WUA", "YRA",
];

use crate::utl8::ASM::*;

use crate::sty2::ev_scurv2;

pub fn check_ev_curv(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let assu = arif.assumption();
    //let yrst = assu.v(PRJ_START_YEAR) as usize;
    let yrst = 2021;
    //let yred = assu.v(PRJ_END_YEAR) as usize;
    let yred = 2050;
    println!("VW:{vnm} PROJ: st:{yrst} ed:{yred}");
    let evsc = ev_scurv2(yrst, yred);
    let at23 = assu.v(EV_AT_2050);
    println!(" assu bat : {}", assu.v(EV_AT_2050));
    println!(" assu ev2030 : {at23}");
    for (i, sc) in evsc.iter().enumerate() {
        let yr = i + yrst;
        let no = sc * at23;
        println!("EVSC: {yr}. {sc} = {no}");
    }
    Ok(())
}

pub fn sub_rpf_test(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let assu = arif.assumption();

    let yrst = assu.u(PRJ_START_YEAR);
    let yred = assu.u(PRJ_END_YEAR);
    println!("VW:{vnm} PROJ: st:{yrst} ed:{yred}");

    let dnm = arif.ass.t(OUTDIR);
    let fnm = format!("{dnm}/asssum-{:?}.bin", SumSub);
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {dnm}/{fnm}").into());
    };
    let Ok((assv0, _)): Result<(Vec<(PeaAssVar, Vec<usize>)>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    println!("sub: {}", assv0.len());

    let mut sb_sub = HashMap::<String, usize>::new();
    for (ii, ass) in assv0.iter().enumerate() {
        let sbid = ass.0.sbid.clone();
        sb_sub.entry(sbid).or_insert(ii);
    }

    println!("====== RPF =====");
    let mut cn = 0;
    for (i, sb) in SUB_RPF.iter().enumerate() {
        let sb = sb.to_string();
        if let Some(ai) = sb_sub.get(&sb) {
            let sola = assv0[*ai].0.v[VarType::SolarEnergy as usize].v;
            let no = if sola > 0.0 {
                cn += 1;
                format!("{cn}")
            } else {
                "".to_string()
            };
            println!("{i}.{sb}: {sola} : {no}");
        } else {
            println!("  NOT FOUND: {sb}");
        }
    }

    println!("====== OVL =====");
    let mut cn = 0;
    for (i, sb) in SUB_OVLD.iter().enumerate() {
        let sb = sb.to_string();
        if let Some(ai) = sb_sub.get(&sb) {
            let sola = assv0[*ai].0.v[VarType::SolarEnergy as usize].v;
            let no = if sola > 0.0 {
                cn += 1;
                format!("{cn}")
            } else {
                "".to_string()
            };
            println!("{i}.{sb}: {sola} : {no}");
        } else {
            println!("  NOT FOUND: {sb}");
        }
    }

    println!("====== RPF + OVL =====");
    let mut cn = 0;
    for (i, sb) in SUB_RPF_OVLD.iter().enumerate() {
        let sb = sb.to_string();
        if let Some(ai) = sb_sub.get(&sb) {
            let sola = assv0[*ai].0.v[VarType::SolarEnergy as usize].v;
            let no = if sola > 0.0 {
                cn += 1;
                format!("{cn}")
            } else {
                "".to_string()
            };
            println!("{i}.{sb}: {sola} : {no}");
        } else {
            println!("  NOT FOUND: {sb}");
        }
    }

    let mut cn = 0;
    for ass in assv0.iter() {
        let sbid = ass.0.sbid.clone();
        let sola = ass.0.v[VarType::SolarEnergy as usize].v;
        if sola == 0.0 {
            continue;
        }
        let rpf = if SUB_RPF.contains(&sbid.as_str()) {
            1
        } else {
            0
        };
        let ovl = if SUB_OVLD.contains(&sbid.as_str()) {
            1
        } else {
            0
        };
        let rpov = if SUB_RPF_OVLD.contains(&sbid.as_str()) {
            1
        } else {
            0
        };
        let all = rpf + ovl + rpov;
        if all > 0 {
            cn += 1;
            println!("{cn}.{sbid} rpf:{rpf} ovl:{ovl} rpov:{rpov}: sola:{sola}");
        }
    }

    Ok(())
}

use crate::utl::load_xlsx;
use bincode::Decode;
use bincode::Encode;

#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct SubRpfOvl {
    pub arid: String,
    pub pvid: String,
    pub sbid: String,
    pub case: String,
    pub year: String,
    pub rpf_mwh: f32,
    pub rpf_fre: i32,
    pub rpf_max: f32,
    pub rpf_avg: f32,
    pub ovl_mwh: f32,
    pub ovl_fre: i32,
    pub ovl_max: f32,
    pub ovl_avg: f32,
    pub pfac: f32,
    pub size: f32,
}

pub const SUB_RPF_OVL_FILE: &str = "/mnt/e/CHMBACK/pea-data/pea2/sub_rfp_ovl.bin";

pub fn sub_excel_rpf_ovl(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let dnm = arif.ass.t(OUTDIR);
    println!("DNM:{dnm}");
    let fnm = "/mnt/e/CHMBACK/pea-data/pea2/pea_sub_rpw_ovl.xlsx";
    let xls = load_xlsx(&vec![fnm])?;
    let mut v_rpfovl = Vec::<SubRpfOvl>::new();
    for (s, sh) in xls.iter().enumerate() {
        println!("nm:{}", sh.shnm);
        for (i, rw) in sh.data.iter().enumerate() {
            if i < 2 {
                continue;
            }
            let arid = rw[0].to_string();
            let pvid = rw[1].to_string();
            let sbid = rw[2].to_string();
            let case = rw[3].to_string();
            let year = rw[4].to_string();
            let (mut rpf_mwh, mut rpf_fre, mut rpf_max, mut rpf_avg) = (0f32, 0i32, 0f32, 0f32);
            let (mut ovl_mwh, mut ovl_fre, mut ovl_max, mut ovl_avg) = (0f32, 0i32, 0f32, 0f32);
            let (mut pfac, mut size) = (0f32, 0f32);
            if s == 0 {
                rpf_mwh = rw[5].parse::<f32>().unwrap_or(0f32);
                rpf_fre = rw[6].parse::<i32>().unwrap_or(0i32);
                rpf_max = rw[7].parse::<f32>().unwrap_or(0f32);
                rpf_avg = rw[8].parse::<f32>().unwrap_or(0f32);
                pfac = rw[9].parse::<f32>().unwrap_or(0f32);
                size = rw[10].parse::<f32>().unwrap_or(0f32);
            }
            if s == 1 {
                ovl_mwh = rw[5].parse::<f32>().unwrap_or(0f32);
                ovl_fre = rw[6].parse::<i32>().unwrap_or(0i32);
                ovl_max = rw[7].parse::<f32>().unwrap_or(0f32);
                ovl_avg = rw[8].parse::<f32>().unwrap_or(0f32);
                pfac = rw[9].parse::<f32>().unwrap_or(0f32);
                size = rw[10].parse::<f32>().unwrap_or(0f32);
            }
            if s == 2 {
                rpf_mwh = rw[5].parse::<f32>().unwrap_or(0f32);
                rpf_fre = rw[6].parse::<i32>().unwrap_or(0i32);
                rpf_avg = rw[7].parse::<f32>().unwrap_or(0f32);
                ovl_mwh = rw[8].parse::<f32>().unwrap_or(0f32);
                ovl_fre = rw[9].parse::<i32>().unwrap_or(0i32);
                ovl_avg = rw[10].parse::<f32>().unwrap_or(0f32);
                pfac = rw[11].parse::<f32>().unwrap_or(0f32);
                size = rw[12].parse::<f32>().unwrap_or(0f32);
            }
            let rpfovl = SubRpfOvl {
                arid,
                pvid,
                sbid,
                case,
                year,
                rpf_mwh,
                rpf_fre,
                rpf_max,
                rpf_avg,
                ovl_mwh,
                ovl_fre,
                ovl_max,
                ovl_avg,
                pfac,
                size,
            };
            //println!("{i}. {rpfovl:?}");
            v_rpfovl.push(rpfovl);
        }
    }
    if let Ok(bin) = bincode::encode_to_vec(&v_rpfovl, bincode::config::standard()) {
        println!("FNM: {SUB_RPF_OVL_FILE} bin:{}", bin.len());
        if std::fs::write(SUB_RPF_OVL_FILE, bin).is_ok() {
            println!("WRITE {SUB_RPF_OVL_FILE}");
        }
    }
    let vv = get_sub_rpf_ovl()?;
    println!("vv:{}", vv.len());
    Ok(())
}

pub fn get_sub_rpf_ovl() -> Result<Vec<SubRpfOvl>, Box<dyn Error>> {
    let bin = std::fs::read(SUB_RPF_OVL_FILE)?;
    let Ok((sub, _)): Result<(Vec<SubRpfOvl>, usize), _> =
        bincode::decode_from_slice(&bin[..], bincode::config::standard())
    else {
        return Err("Can open file".into());
    };
    Ok(sub)
}

pub const BRANCH_INFO2_FILE: &str = "/mnt/e/CHMBACK/pea-data/pea2/branch_info2.bin";
//pub const BRANCH_INFO2_FILE: &str = "/mnt/e/CHMBACK/pea-data/pea2/branch_info2.bin";
use crate::dcl::ProcEngine;
//use crate::utl7::PeaBranch;
use crate::utl7::AOJ_ERR1;
use crate::utl7::BRANCH_PREF_BRN;
use crate::utl7::BRANCH_PREF_PRV;
use sglab02_lib::sg::gis1::ar_list;
use sglib04::geo3::GisAoj;

#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct BranchAoj {
    pub ii: usize,
    pub no: String,
    pub up: String,
    pub name: String,
    pub pai: Option<usize>,
    pub is_prv: bool,
    pub size: String,
    pub has_stock: bool,
    pub i_stock: Option<usize>,
    pub i_prov: Option<usize>,
    pub org: String,
    pub code: String,
    pub chd: Vec<usize>,
    pub stock_for: Vec<usize>,
    pub prov_memb: Vec<usize>,

    pub ar: String,
    pub xmin: Option<f32>,
    pub ymin: Option<f32>,
    pub xmax: Option<f32>,
    pub ymax: Option<f32>,
    pub level: Option<f32>,
    pub center_x: Option<f32>,
    pub center_y: Option<f32>,
    pub sht_name: Option<String>,
    pub shp_len: Option<f32>,
    pub office: Option<String>,
    pub parent1: Option<String>,
    pub parent2: Option<String>,
    pub pea: Option<String>,
    pub ar_cd: Option<String>,
    pub shp_area: Option<f32>,
    pub prv_cd: Option<String>,
    pub aoj_sz: Option<String>,
    pub reg: Option<String>,
    pub gons: Vec<Vec<(f32, f32)>>,
}

#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct GoogleMapInfo {
    pub mg: u32,
    pub updw: u32,
    pub ww: f32,
    pub hh: f32,
    pub ex_x: f32,
    pub ex_y: f32,
    pub or_x: f32,
    pub or_y: f32,
    pub ofs_x: f32,
    pub w: f32,
    pub xx: f32,
    pub yy: f32,
    pub zm: u32,
}

pub fn a_tee_req_01(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let dnm = arif.ass.t(OUTDIR);
    let fnm = format!("asssum-{:?}.bin", AssSumEnum::SumSub);
    let prv_map_dir = format!("{dnm}/tee_map1");
    std::fs::create_dir_all(&prv_map_dir)?;

    let Ok(buf) = std::fs::read(format!("{dnm}/{fnm}")) else {
        return Err(format!("Not found {dnm}/{fnm}").into());
    };
    let Ok((assv0, _)): Result<(Vec<(PeaAssVar, Vec<usize>)>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    let mut assv1 = assv0.clone();
    for ass in assv1.iter_mut() {
        let pwcp = ass.0.v[VarType::SubPowCap as usize].v;
        let trst = ass.0.v[VarType::PowTrSat as usize].v;
        //let mvst = ass.0.v[VarType::MvPowSatTr as usize].v;
        let rt02 = pwcp * (1.0 - trst);
        let rt02 = if pwcp>=100.0 { rt02 } else { 0.0 };
        let rt02 = if trst>0.0 && trst<0.3 { rt02 } else { 0.0 };
        ass.0.v[VarType::TakeNote as usize].v = rt02;
    }
    assv1.sort_by(|a,b| { let a0 = a.0.v[VarType::TakeNote as usize].v; let b0 = b.0.v[VarType::TakeNote as usize].v; a0.partial_cmp(&b0).unwrap() });
    println!("sub: {}", assv1.len());

    use crate::dcl::Geo;

    let mut pv_cnt = HashMap::<String,usize>::new();
    let mut pv_ass = HashMap::<String,Vec<PeaAssVar>>::new();
    for ass in assv1.iter() {
        let sbid = ass.0.sbid.to_string();
        let pvid = ass.0.pvid.to_string();
        let pwcp = ass.0.v[VarType::SubPowCap as usize].v;
        let trst = ass.0.v[VarType::PowTrSat as usize].v;
        let mvst = ass.0.v[VarType::MvPowSatTr as usize].v;
        //let rt01 = if mvst==0.0 { 0.0 } else { trst/mvst };
        let rt02 = pwcp * (1.0 - trst);
        let rt03 = ass.0.v[VarType::TakeNote as usize].v;
        if rt03==0.0 { continue; }

        let vass = pv_ass.entry(pvid.clone()).or_default();
        vass.push(ass.0.clone());
        let (x,y) = ass.0.n1d.n1d_2_latlon();
        let cnt = pv_cnt.entry(pvid.clone()).or_default();
        *cnt += 1;
        println!("{sbid} {pvid} pw:{pwcp} t:{trst} m:{mvst} r:{rt02} : n:{rt03} {x},{y}");
    }
    println!("CNT:{} ASS:{}", pv_ass.len(), pv_cnt.len());
    for (k,v) in pv_cnt.iter() {
        println!("{k} = {v}");
    }

    let gis_out = "/mnt/e/CHMBACK/pea-data/GISDB";
    let gis_dir = "/mnt/e/CHMBACK/pea-data/GIS/GIS-2024-09-12";
    let ly = "LB_Changwat";

    println!("read gis1 {gis_dir}");
    let ars = ar_list();
    let mut v_pvdb = vec![];
    let mut v_pvpo = vec![];
    for r in &ars {
        std::fs::create_dir_all(gis_out).expect("ERR");
        let rgf = format!("{gis_dir}/{r}/{ly}.dbf");
        //println!("rgf {}", rgf);
        let mut reader = dbase::Reader::from_path_with_encoding(rgf.clone(),CP874).unwrap();
        for rc in reader.iter_records() {
            let rc = rc.unwrap();
            let r = db_rec(rc.clone());
            v_pvdb.push(r);
            //vdb.push(r);
        }
        let rgf = format!("{gis_dir}/{r}/{ly}.shp");
        if let Ok(mut reader) = shapefile::Reader::from_path(rgf.clone()) {
            for (gon, _rc) in reader.iter_shapes_and_records_as::<shapefile::Polygon, dbase::Record>() .flatten() {
                let mut ringpnts = Vec::<Vec<(f64, f64)>>::new();
                for ring in gon.into_inner() {
                    let mut pnts = Vec::<(f64, f64)>::new();
                    for pnt in ring.into_inner() {
                        pnts.push((pnt.x, pnt.y));
                    }
                    ringpnts.push(pnts);
                }
                v_pvpo.push(ringpnts);
            }
        }
    }
    println!("db: {} {}", v_pvdb.len(), v_pvpo.len());

    let mut pv_map = HashMap::<String,(HashMap<String,DbfVal>, Vec<Vec<(f64,f64)>>)>::new();
    for (po,db) in v_pvpo.iter().zip(v_pvdb.iter()) {
        let Some(DbfVal::Character(Some(nm))) = db.get("CHANGWAT_N") else {
            println!("NO FIELD CHANGWAT_N");
            continue;
        };
        if pv_map.contains_key(nm) { continue; }
        pv_map.insert(nm.clone(), (db.clone(), po.clone()));
    }
    let mut workbook = Workbook::new();
    let sht = workbook.add_worksheet();
    let _ = sht.set_name("Substation");
    sht.set_column_width(0, 5)?;
    sht.set_column_width(1, 20)?;
    sht.set_column_width(2, 15)?;
    sht.set_column_width(3, 15)?;
    sht.set_column_width(4, 25)?;
    /*
    let ttfm = Format::new().set_bold();
    */
    let hdfm = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xC6EFCE));
    let dtfm = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xEFCEC6));

    sht.write_with_format(1, 0, "NO", &hdfm)?;
    sht.write_with_format(1, 1, "PROVINCE", &hdfm)?;
    sht.write_with_format(1, 2, "SUBSTATION", &hdfm)?;
    sht.write_with_format(1, 3, "AVAIL: MW", &hdfm)?;
    sht.write_with_format(1, 4, "LOCATION", &hdfm)?;

    let mut cn = 0;
    let rw = 1;
    for (pv,assv) in pv_ass.iter() {
        for ass in assv.iter() {
            cn += 1;
            let pv = pv.to_string();
            let sb = ass.sbid.to_string();
            let tn = ass.v[VarType::TakeNote as usize].v.pan(0);
            let pw = ass.v[VarType::SubPowCap as usize].v.pan(0);
            let tn = format!("{tn}/{pw}");
            let (x,y) = ass.n1d.n1d_2_latlon();
            let ll = format!("{x},{y}");
            sht.write_with_format(rw+cn, 0, cn, &dtfm)?;
            sht.write_with_format(rw+cn, 1, pv, &dtfm)?;
            sht.write_with_format(rw+cn, 2, sb, &dtfm)?;
            sht.write_with_format(rw+cn, 3, tn, &dtfm)?;
            sht.write_with_format(rw+cn, 4, ll, &dtfm)?;
        }
    }

    let ftn = format!("./temp/sub_empty_2.xlsx");
    workbook.save(ftn)?;

    /*
    let mut cn1 = 0;
    for (nm,(_db,po)) in pv_map.iter() {
        if let Some(cn) = pv_cnt.get(nm) {
            if *cn==0 { continue; }
        } else {
            continue;
        }
        let mut pns = vec![];
        for po in po.iter() {
            for (x, y) in po.iter() {
                pns.push((*x as f32, *y as f32));
            }
        }
        cn1 += 1;
        println!("{cn1}: {nm}");
        let mut pv_ply = HashMap::<String,Vec<Vec<(f64,f64)>>>::new();
        pv_ply.insert(nm.clone(), po.clone());
        let map = get_ggmap_info(pns);
        let fmap1 = format!("{prv_map_dir}/{}-m1.jpeg", nm);
        let fmap2 = format!("{prv_map_dir}/{}-m2.jpeg", nm);
        let fmap3 = format!("{prv_map_dir}/{}-m3.png", nm);
        let fmap4 = format!("{prv_map_dir}/{}-m4.jpeg", nm);
        load_ggmap(&map, "roadmap", fmap1.as_str());
        let fims = vec![fmap1,fmap2,fmap3,fmap4];
        draw_tee_1(&map, pv_ply, pv_ass.clone(), fims);
    }
    */
    Ok(())
}

pub fn draw_tee_1(map: &GoogleMapInfo, pv_ply: HashMap<String,Vec<Vec<(f64,f64)>>>, pv_ass: HashMap<String,Vec<PeaAssVar>>, fims: Vec<String>) {
    let fim1 = fims[0].as_str();
    let fim2 = fims[1].as_str();
    let fim3 = fims[2].as_str();
    let fim4 = fims[3].as_str();
    if let Ok(img) = ImageReader::open(fim1) && let Ok(mut img) = img.decode() {
        let (w, h) = (img.width(), img.height());
        let img = img.crop(map.mg, map.updw, w - map.mg, h - map.updw * 2);
        //let frdcp = format!("{}cp.jpeg", fpath);
        img.save(fim2).expect("?");

        let (w, h) = (img.width(), img.height());

        use std::fmt::Write;
        let mut jsmp = String::new();
        writeln!(jsmp, "{{ \"map\": [").expect("?");

        let (t_x, t_y) = (0, 0);
        let fcx = map.ww / map.ex_x;
        let fcy = map.ww / map.ex_x;
        let mut img0 = RgbaImage::new(w, h);

        let ffx = 1.00;
        let ffy = 1.00;
        let mut ks = vec![];
        for (k,ply) in pv_ply.iter() {
            ks.push(k.clone());
            for po in ply.iter() {
                let mut v_pn = vec![];
                let mut v_pf = vec![];
                for (x,y) in po.iter() {
                    let x = (*x as f32 - map.or_x) * fcx * ffx - map.ofs_x;
                    let y = (*y as f32 - map.or_y) * fcy * ffy;
                    let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
                    let pf = Point { x:ix as f32, y:iy as f32 };
                    let pn = Point { x:ix, y:iy };
                    if !v_pn.is_empty() && v_pn[v_pn.len()-1]==pn { continue; }
                    v_pn.push(pn);
                    v_pf.push(pf);
                }
                v_pn.pop();
                v_pf.pop();
                draw_polygon_mut(&mut img0, &v_pn, Rgba([0,200,0,30]));
                draw_hollow_polygon_mut(&mut img0, &v_pf, Rgba([0, 0, 0, 255]));
            }
        }
        println!("k prv: {ks:?}");
        if ks.is_empty() { return; }
        let p = ks[0].to_string();
        let Some(assv) = pv_ass.get(&p) else {
            return;
        };
        println!("member {}", assv.len());
        for a in assv.iter() {
            let (x,y) = a.n1d.n1d_2_utm();
            println!("  === {x},{y}");
            let x = (x as f32 - map.or_x) * fcx * ffx - map.ofs_x;
            let y = (y as f32 - map.or_y) * fcy * ffy;
            let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
            //let pf = Point { x:ix as f32, y:iy as f32 };
            let pn1 = Point { x:ix, y:iy-20 };
            let pn2 = Point { x:ix-18, y:iy+14 };
            let pn3 = Point { x:ix+18, y:iy+14 };
            let pns = vec![pn1,pn2,pn3];
            draw_polygon_mut(&mut img0, &pns, Rgba([180,0,0,200]));
        }
        //println!("fim3: {fim3}");

        img0.save(fim3).expect("?");

        let mut img1 = open(fim2).unwrap();
        let img2 = open(fim3).unwrap();
        img1.blend(&img2, pixel_mult, true, false).unwrap();
        img1.save(fim4).expect("?");
    }
}


pub fn get_ggmap_info(pns: Vec<(f32,f32)>) -> GoogleMapInfo {

        let mg = MP_MG;
        let updw = MP_UPDW;
        let ww = MP_WW;
        let hh = MP_HH;

        let (w, _h) = (mg as f32 + ww, updw as f32 * 2.0 + hh);
        //pub gons: Vec<Vec<(f32, f32)>>,

        let fst = (pns[0].0, pns[1].1);
        let (mut x0, mut y0, mut x1, mut y1) = (fst.0, fst.1, fst.0, fst.1);
        for pnt in pns.iter() {
            x0 = x0.min(pnt.0);
            y0 = y0.min(pnt.1);
            x1 = x1.max(pnt.0);
            y1 = y1.max(pnt.1);
        }
        let (ox, oy) = ((x1 + x0) * 0.5f32, (y1 + y0) * 0.5f32);
        let wd = x1 - x0;
        let (o_ld, _o_ln) = utm_latlong(ox, oy);
        let zm = meter_pixel_to_zoom_lat_2(wd, ww as u32, o_ld);
        let mtpx = zoom_to_meter_pixel_lat(zm, o_ld);
        let ex_x = mtpx * ww;
        let ex_y = mtpx * hh;
        let (o_x, o_y) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        let (xx, yy) = utm_latlong(o_x, o_y);
        let or_x = o_x - ex_x / 2.0;
        let or_y = o_y - ex_y / 2.0;
        let ofs_x = 40f32;
        GoogleMapInfo {
            mg,
            updw,
            ww,
            hh,
            ex_x,
            ex_y,
            or_x,
            or_y,
            ofs_x,
            w,
            xx,
            yy,
            zm,
        }
}

use crate::dcl::Geo;
use sglib04::geo1::MeterBill;

#[derive(Debug, Default, Clone)]
pub struct BillCate {
    pub cnt: usize,
    pub a15: f32,
    pub a18: f32,
    pub a19: f32,
    pub c200: usize,
    pub a200: f32,
    pub c5k: usize,
    pub a5k: f32,
}

pub fn a_tee_req_03(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    //let dnm = arif.ass.t(OUTDIR);
    let fdir = "/mnt/e/CHMBACK/pea-data/20240801_กรอ";
    let fout = "/mnt/e/CHMBACK/pea-data/data1";
    //let flst = vec!["202402", "202405"];
    let flst = vec!["202405"];
    //let _csv_v = Vec::<CSVFile>::new();
    //let mut rate = HashMap::<String, (usize,f32,f32,f32)>::new();
    let mut rate = HashMap::<String, BillCate>::new();
    let mut volt = HashMap::<String, usize>::new();
    let mut er2 = 0;
    //let mut cn = 0;
    for f in flst {
        let fln = format!("{fdir}/export_กรอ_bil013_{}.csv", f);
        let fou = format!("{fout}/{}_bil.bin", f);
        println!("start {}", &fln);
        if let Ok(mut rdr) = csv::Reader::from_path(&fln) {
            let mut bils = Vec::<MeterBill>::new();
            for rc in rdr.records().flatten() {
                // if the record exist
                if let (
                    Some(c0),
                    Some(c2),
                    Some(c3),
                    Some(c4),
                    Some(c5),
                    Some(c6),
                    Some(c7),
                    Some(c8),
                    Some(c9),
                    Some(c15),
                    Some(c18),
                    Some(c19),
                ) = (
                    rc.get(0),
                    rc.get(2),
                    rc.get(3),
                    rc.get(4),
                    rc.get(5),
                    rc.get(6),
                    rc.get(7),
                    rc.get(8),
                    rc.get(9),
                    rc.get(15),
                    rc.get(18),
                    rc.get(19),
                ) {
                    let n15 = c15.parse::<f32>().unwrap_or_default();
                    let n18 = c18.parse::<f32>().unwrap_or_default();
                    let n19 = c19.parse::<f32>().unwrap_or_default();
                    if let Some(vo) = volt.get_mut(c5) {
                        *vo += 1;
                    } else {
                        volt.insert(c5.to_string(), 1);
                    }
                    let rt = rate.entry(c4.to_string()).or_default();
                    rt.cnt += 1;
                    rt.a15 += n15;
                    rt.a18 += n18;
                    rt.a19 += n19;
                    if n18<=200.0 {
                        rt.c200 += 1;
                        rt.a200 += n18;
                    }
                    if n19>=5000.0 {
                        rt.c5k += 1;
                        rt.a5k += n19;
                    }
                    /*

                    if let Some((rt,a15,a18,a19)) = rate.get_mut(c4) {
                        *rt += 1;
                        *a15 += n15;
                        *a18 += n18;
                        *a19 += n19;
                    } else {
                        rate.insert(c4.to_string(), (1usize,n15,n18,n19));
                    }
                    */
                    let mb0 = MeterBill {
                        trsg: c0.trim().to_string(),
                        pea: c7.trim().to_string(),
                        ca: c2.trim().to_string(),
                        inst: c3.trim().to_string(),
                        rate: c4.to_string(),
                        volt: c5.to_string(),
                        mru: c6.to_string(),
                        mat: c8.trim().to_string(),
                        main: c9.trim().to_string(),
                        kwh15: n15,
                        kwh18: n18,
                        amt19: n19,
                        ..Default::default()
                    };
                    if !c9.is_empty() {
                        //println!("c9: {c9} {} {}", mb0.kwh15, mb0.amt19);
                    }
                    bils.push(mb0);
                } else {
                    er2 += 1;
                }
            } // loop all rec
            println!("er2:{er2}");
            let mut keys = rate.iter().map(|(k,_)| k.to_string()).collect::<Vec<_>>();
            //let mut keys = rate.keys().map(|k| k.to_string());
            keys.sort();
            for k in keys {
                //let Some((cn,n15,n18,n19)) = rate.get(&k) else {
                let Some(bc) = rate.get(&k) else {
                    continue;
                };
                let cn = bc.cnt;
                let n18 = bc.a18;
                let n19 = bc.a19;
                let c200 = bc.c200;
                let a200 = bc.a200;
                let c5k = bc.c5k;
                let a5k = bc.a5k;
                println!("{k} {cn} {n18} {n19} {c200} {a200} {c5k} {a5k}");
            }
            println!("rate: {rate:?}");
            println!("volt: {volt:?}");
            println!("write {fou}");
            println!("writen {fln}");
        }
    }
     Ok(())
}

pub fn a_tee_req_02(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let dnm = arif.ass.t(OUTDIR);
    let fnm = format!("asssum-{:?}.bin", AssSumEnum::SumSub);

    let Ok(buf) = std::fs::read(format!("{dnm}/{fnm}")) else {
        return Err(format!("Not found {dnm}/{fnm}").into());
    };
    let Ok((assv0, _)): Result<(Vec<(PeaAssVar, Vec<usize>)>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    let mut assv1 = assv0.clone();
    for ass in assv1.iter_mut() {
        let pwcp = ass.0.v[VarType::SubPowCap as usize].v;
        //let trst = ass.0.v[VarType::PowTrSat as usize].v;
        //let mvst = ass.0.v[VarType::MvPowSatTr as usize].v;
        if pwcp<100.0 {
            ass.0.v[VarType::SolarEnergy as usize].v = 0.0;
        }
    }
    //assv1.sort_by(|a,b| { let a0 = a.0.v[VarType::TakeNote as usize].v; let b0 = b.0.v[VarType::TakeNote as usize].v; a0.partial_cmp(&b0).unwrap() });
    assv1.sort_by(|a,b| {
        let a0 = a.0.v[VarType::SolarEnergy as usize].v;
        let b0 = b.0.v[VarType::SolarEnergy as usize].v;
        a0.partial_cmp(&b0).unwrap() });
    println!("sub: {}", assv1.len());


    for ass in assv1.iter() {
        let sbid = ass.0.sbid.to_string();
        let pvid = ass.0.pvid.to_string();
        let pwcp = ass.0.v[VarType::SubPowCap as usize].v;
        let trst = ass.0.v[VarType::PowTrSat as usize].v;
        let mvst = ass.0.v[VarType::MvPowSatTr as usize].v;
        //let rt01 = if mvst==0.0 { 0.0 } else { trst/mvst };
        let rt02 = pwcp * (1.0 - trst);
        let rt03 = ass.0.v[VarType::SolarEnergy as usize].v;
        let (x,y) = ass.0.n1d.n1d_2_latlon();
        println!("{sbid} {pvid} pw:{pwcp} t:{trst} m:{mvst} r:{rt02} : n:{rt03} {x},{y}");
    }
    //let mut workbook = Workbook::new();
    Ok(())
}

pub const PROV_SET1: [&str; 4] = [
    "เชียงใหม่",
    "ชลบุรี",
    "นครราชสีมา",
    "สุราษฎร์ธานี",
];

pub fn apao_req_01(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let dnm = arif.ass.t(OUTDIR);
    let prv_map_dir = format!("{dnm}/map4");
    std::fs::create_dir_all(&prv_map_dir)?;

    // ============ BRANCH POLYGON BEGIN ==========
    let fnm = BRANCH_INFO2_FILE;
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((aojv, _)): Result<(Vec<BranchAoj>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("!!!! ERROR #7 Not decoded".into());
    };
    println!("============== AOJ {}", aojv.len());
    let mut cd_aoj = HashMap::<String,BranchAoj>::new();
    for aoj in aojv.into_iter() {
        cd_aoj.insert(aoj.code.clone(), aoj);
    }
    println!("============== AOJ HM {}", cd_aoj.len());
    // ============ BRANCH POLYGON BEGIN ==========

    // ============ SELECT 2 BEGIN ==========
    let fnm = format!("{dnm}/asssum-{:?}.bin", SumPrvBrn1);
    println!("TASK2 DNM:{fnm}");
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((mut assv0, _)): Result<(Vec<(PeaAssVar, Vec<usize>)>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    assv0.sort_by(|a, b| {
        let a0 = a.0.v[VarType::FirSum.tousz()].v;
        let b0 = b.0.v[VarType::FirSum.tousz()].v;
        b0.partial_cmp(&a0).unwrap()
    });
    println!("assv: {}", assv0.len());
    // ============ SELECT 2 BEGIN ==========

    // ============ PROV BRANCH BEGIN ==========
    let fnm = format!("{dnm}/asssum-{:?}-m.bin", SumPrvBrn1);
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((pvassm, _)): Result<(HashMap<String,Vec<String>>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    // ============ PROV BRANCH END ==========

    // ============ PROV BRANCH MAIN BEGIN ==========
    let fnm = format!("{dnm}/asssum-{:?}-b.bin", SumPrvBrn1);
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((brn_mb, _)): Result<(HashMap<String,Vec<String>>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    println!("====== BRN MB: {}", brn_mb.len());

    // ============ PROV BRANCH MAIN END ==========
    /*
    println!("pvassm: {}", pvassm.len());
    for (k,v) in pvassm.iter() {
        println!("BRN MB: {k} {v:?}");
    }
    */
    for pvas in assv0.iter() {
        let Some(mb) = pvassm.get(&pvas.0.pvid) else {
            continue;
        };
        println!("{} - mb:{} tr:{}", pvas.0.pvid, mb.len(), pvas.0.v[VarType::NoPeaTr as usize].v);
    }
    // ============ PROV BRANCH BEGIN ==========

    let gis_out = "/mnt/e/CHMBACK/pea-data/GISDB";
    let gis_dir = "/mnt/e/CHMBACK/pea-data/GIS/GIS-2024-09-12";
    let ly = "LB_Changwat";
        //let rg = "LB_AOJ";

    println!("read gis1 {gis_dir}");
    let ars = ar_list();
    let mut v_pvdb = vec![];
    let mut v_pvpo = vec![];
    for r in &ars {
        std::fs::create_dir_all(gis_out).expect("ERR");
        let rgf = format!("{gis_dir}/{r}/{ly}.dbf");
        println!("rgf {}", rgf);
        let mut reader = dbase::Reader::from_path_with_encoding(rgf.clone(),CP874).unwrap();
        for rc in reader.iter_records() {
            let rc = rc.unwrap();
            let r = db_rec(rc.clone());
            v_pvdb.push(r);
            //vdb.push(r);
        }
        let rgf = format!("{gis_dir}/{r}/{ly}.shp");
        if let Ok(mut reader) = shapefile::Reader::from_path(rgf.clone()) {
            for (gon, _rc) in reader.iter_shapes_and_records_as::<shapefile::Polygon, dbase::Record>() .flatten() {
                let mut ringpnts = Vec::<Vec<(f64, f64)>>::new();
                for ring in gon.into_inner() {
                    let mut pnts = Vec::<(f64, f64)>::new();
                    for pnt in ring.into_inner() {
                        pnts.push((pnt.x, pnt.y));
                    }
                    ringpnts.push(pnts);
                }
                v_pvpo.push(ringpnts);
            }
        }
    }
    println!("db: {} {}", v_pvdb.len(), v_pvpo.len());

    let mut pv_po_db = HashMap::<String,_>::new();
    for (po,db) in v_pvpo.iter().zip(v_pvdb.iter()) {
        let Some(DbfVal::Character(Some(nm))) = db.get("CHANGWAT_N") else {
            println!("NO FIELD CHANGWAT_N");
            continue;
        };
        //println!("PV: {nm}");
        pv_po_db.insert(nm.clone(), (po.clone(), db.clone()));
    }
    println!("pv_po_db: {}", pv_po_db.len());

    for (ass,_) in assv0.iter() {
        let Some((po,_db)) = pv_po_db.get(&ass.pvid) else {
            println!("!!!!! ERROR  {}", ass.pvid);
            continue;
        };
        //println!(">>>>>>>>>>>>  {}", ass.pvid);
        /*
        if !PROV_SET1.contains(&ass.pvid.as_str()) {
            continue;
        }

        let mg = MP_MG;
        let updw = MP_UPDW;
        let ww = MP_WW;
        let hh = MP_HH;

        let (w, _h) = (mg as f32 + ww, updw as f32 * 2.0 + hh);
        */
        //pub gons: Vec<Vec<(f32, f32)>>,

        let mut pns = vec![];
        for po in po.iter() {
            for (x, y) in po.iter() {
                pns.push((*x as f32, *y as f32));
            }
        }
        if pns.is_empty() {
            println!("!!!! BRANCH {} is empty polygon", ass.pvid);
            continue;
        }

        //println!("ALL BRANCH {}", pvassm.len());
        let Some(brns) = brn_mb.get(&ass.pvid) else {
            println!("ERROR #6 PROVINCE NOT FOUND {}", ass.pvid);
            continue;
        };
        //println!(" BRANCH {}", brns.len());

        //println!("PRV: {}", ass.pvid);
        /*
        println!("BRANCH OK: {} - {}", ass.pvid, pns.len());
        let fst = (pns[0].0, pns[1].1);
        let (mut x0, mut y0, mut x1, mut y1) = (fst.0, fst.1, fst.0, fst.1);
        for pnt in pns.iter() {
            x0 = x0.min(pnt.0);
            y0 = y0.min(pnt.1);
            x1 = x1.max(pnt.0);
            y1 = y1.max(pnt.1);
        }
        let (ox, oy) = ((x1 + x0) * 0.5f32, (y1 + y0) * 0.5f32);
        let wd = x1 - x0;
        let (o_ld, _o_ln) = utm_latlong(ox, oy);
        let zm = meter_pixel_to_zoom_lat_2(wd, ww as u32, o_ld);
        let mtpx = zoom_to_meter_pixel_lat(zm, o_ld);
        let ex_x = mtpx * ww;
        let ex_y = mtpx * hh;
        let (o_x, o_y) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        let (xx, yy) = utm_latlong(o_x, o_y);
        let or_x = o_x - ex_x / 2.0;
        let or_y = o_y - ex_y / 2.0;
        let ofs_x = 40f32;
        */
        /*
        let map = GoogleMapInfo {
            mg,
            updw,
            ww,
            hh,
            ex_x,
            ex_y,
            or_x,
            or_y,
            ofs_x,
            w,
            xx,

            yy,
            zm,
        };
        */
        //let nm = ass.pvid.clone();
        //let fmap1 = format!("{prv_map_dir}/{}-m1.jpeg", nm);
        //let fmap2 = format!("{prv_map_dir}/{}-m2.jpeg", nm);
        //let fmap3 = format!("{prv_map_dir}/{}-m3.png", nm);
        //let fmap4 = format!("{prv_map_dir}/{}-m4.jpeg", nm);
        //load_ggmap(&map, "roadmap", fmap1.as_str());
        //let mut grps = vec![];
        let mut mcn = 0;
        for brn in brns.iter() {
            let Some(_chds) = pvassm.get(brn) else {
                println!("  ERROR #7 no child {brn}");
                continue;
            };
            //let nn = chds.len();
            //println!("  MAIN: {brn}: {nn}");
            mcn += 1;

            /*
            let mut grp = vec![];
            for (i,chd) in chds.iter().enumerate() {
                println!("     {i}: {chd}");
                let Some(br) = cd_aoj.get(chd) else {
                    continue;
                };
                let mut off = vec![];
                for gon in br.gons.iter() {
                    let mut lns = vec![];
                    for (x,y) in gon.iter() {
                        lns.push((*x, *y));
                    }
                    off.push(lns);
                }
                grp.push(off);
            }
            grps.push(grp);
            */
        }
        println!("{} {mcn}", ass.pvid);
        //let fims = vec![fmap1,fmap2,fmap3,fmap4];
        //draw_ass_map4(&map, po, &grps, fims);
    }
     Ok(())
}


pub fn draw_brn_task4(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let dnm = arif.ass.t(OUTDIR);
    let prv_map_dir = format!("{dnm}/map4");
    std::fs::create_dir_all(&prv_map_dir)?;

    // ============ BRANCH POLYGON BEGIN ==========
    let fnm = BRANCH_INFO2_FILE;
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((aojv, _)): Result<(Vec<BranchAoj>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("!!!! ERROR #7 Not decoded".into());
    };
    println!("============== AOJ {}", aojv.len());
    let mut cd_aoj = HashMap::<String,BranchAoj>::new();
    for aoj in aojv.into_iter() {
        cd_aoj.insert(aoj.code.clone(), aoj);
    }
    println!("============== AOJ HM {}", cd_aoj.len());
    // ============ BRANCH POLYGON BEGIN ==========

    // ============ SELECT 2 BEGIN ==========
    let fnm = format!("{dnm}/asssum-{:?}.bin", SumPrvBrn1);
    println!("TASK2 DNM:{fnm}");
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((assv0, _)): Result<(Vec<(PeaAssVar, Vec<usize>)>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    println!("assv: {}", assv0.len());
    // ============ SELECT 2 BEGIN ==========

    // ============ PROV BRANCH BEGIN ==========
    let fnm = format!("{dnm}/asssum-{:?}-m.bin", SumPrvBrn1);
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((pvassm, _)): Result<(HashMap<String,Vec<String>>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    // ============ PROV BRANCH END ==========

    // ============ PROV BRANCH MAIN BEGIN ==========
    let fnm = format!("{dnm}/asssum-{:?}-b.bin", SumPrvBrn1);
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((brn_mb, _)): Result<(HashMap<String,Vec<String>>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    println!("====== BRN MB: {}", brn_mb.len());

    // ============ PROV BRANCH MAIN END ==========
    /*
    println!("pvassm: {}", pvassm.len());
    for (k,v) in pvassm.iter() {
        println!("BRN MB: {k} {v:?}");
    }
    */
    for pvas in assv0.iter() {
        let Some(mb) = pvassm.get(&pvas.0.pvid) else {
            continue;
        };
        println!("{} - mb:{} tr:{}", pvas.0.pvid, mb.len(), pvas.0.v[VarType::NoPeaTr as usize].v);
    }
    // ============ PROV BRANCH BEGIN ==========

    let gis_out = "/mnt/e/CHMBACK/pea-data/GISDB";
    let gis_dir = "/mnt/e/CHMBACK/pea-data/GIS/GIS-2024-09-12";
    let ly = "LB_Changwat";
        //let rg = "LB_AOJ";

    println!("read gis1 {gis_dir}");
    let ars = ar_list();
    let mut v_pvdb = vec![];
    let mut v_pvpo = vec![];
    for r in &ars {
        std::fs::create_dir_all(gis_out).expect("ERR");
        let rgf = format!("{gis_dir}/{r}/{ly}.dbf");
        println!("rgf {}", rgf);
        let mut reader = dbase::Reader::from_path_with_encoding(rgf.clone(),CP874).unwrap();
        for rc in reader.iter_records() {
            let rc = rc.unwrap();
            let r = db_rec(rc.clone());
            v_pvdb.push(r);
            //vdb.push(r);
        }
        let rgf = format!("{gis_dir}/{r}/{ly}.shp");
        if let Ok(mut reader) = shapefile::Reader::from_path(rgf.clone()) {
            for (gon, _rc) in reader.iter_shapes_and_records_as::<shapefile::Polygon, dbase::Record>() .flatten() {
                let mut ringpnts = Vec::<Vec<(f64, f64)>>::new();
                for ring in gon.into_inner() {
                    let mut pnts = Vec::<(f64, f64)>::new();
                    for pnt in ring.into_inner() {
                        pnts.push((pnt.x, pnt.y));
                    }
                    ringpnts.push(pnts);
                }
                v_pvpo.push(ringpnts);
            }
        }
    }
    println!("db: {} {}", v_pvdb.len(), v_pvpo.len());

    let mut pv_po_db = HashMap::<String,_>::new();
    for (po,db) in v_pvpo.iter().zip(v_pvdb.iter()) {
        let Some(DbfVal::Character(Some(nm))) = db.get("CHANGWAT_N") else {
            println!("NO FIELD CHANGWAT_N");
            continue;
        };
        //println!("PV: {nm}");
        pv_po_db.insert(nm.clone(), (po.clone(), db.clone()));
    }
    println!("pv_po_db: {}", pv_po_db.len());

    for (ass,_) in assv0.iter() {
        let Some((po,_db)) = pv_po_db.get(&ass.pvid) else {
            println!("!!!!! ERROR  {}", ass.pvid);
            continue;
        };
        //println!(">>>>>>>>>>>>  {}", ass.pvid);
        if !PROV_SET1.contains(&ass.pvid.as_str()) {
            continue;
        }

        let mg = MP_MG;
        let updw = MP_UPDW;
        let ww = MP_WW;
        let hh = MP_HH;

        let (w, _h) = (mg as f32 + ww, updw as f32 * 2.0 + hh);
        //pub gons: Vec<Vec<(f32, f32)>>,

        let mut pns = vec![];
        for po in po.iter() {
            for (x, y) in po.iter() {
                pns.push((*x as f32, *y as f32));
            }
        }
        if pns.is_empty() {
            println!("!!!! BRANCH {} is empty polygon", ass.pvid);
            continue;
        }

        println!("ALL BRANCH {}", pvassm.len());
        let Some(brns) = brn_mb.get(&ass.pvid) else {
            println!("ERROR #6 PROVINCE NOT FOUND {}", ass.pvid);
            continue;
        };
        println!(" BRANCH {}", brns.len());

        println!("PRV: {}", ass.pvid);
        println!("BRANCH OK: {} - {}", ass.pvid, pns.len());
        let fst = (pns[0].0, pns[1].1);
        let (mut x0, mut y0, mut x1, mut y1) = (fst.0, fst.1, fst.0, fst.1);
        for pnt in pns.iter() {
            x0 = x0.min(pnt.0);
            y0 = y0.min(pnt.1);
            x1 = x1.max(pnt.0);
            y1 = y1.max(pnt.1);
        }
        let (ox, oy) = ((x1 + x0) * 0.5f32, (y1 + y0) * 0.5f32);
        let wd = x1 - x0;
        let (o_ld, _o_ln) = utm_latlong(ox, oy);
        let zm = meter_pixel_to_zoom_lat_2(wd, ww as u32, o_ld);
        let mtpx = zoom_to_meter_pixel_lat(zm, o_ld);
        let ex_x = mtpx * ww;
        let ex_y = mtpx * hh;
        let (o_x, o_y) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        let (xx, yy) = utm_latlong(o_x, o_y);
        let or_x = o_x - ex_x / 2.0;
        let or_y = o_y - ex_y / 2.0;
        let ofs_x = 40f32;
        let map = GoogleMapInfo {
            mg,
            updw,
            ww,
            hh,
            ex_x,
            ex_y,
            or_x,
            or_y,
            ofs_x,
            w,
            xx,
            yy,
            zm,
        };
        let nm = ass.pvid.clone();
        let fmap1 = format!("{prv_map_dir}/{}-m1.jpeg", nm);
        let fmap2 = format!("{prv_map_dir}/{}-m2.jpeg", nm);
        let fmap3 = format!("{prv_map_dir}/{}-m3.png", nm);
        let fmap4 = format!("{prv_map_dir}/{}-m4.jpeg", nm);
        load_ggmap(&map, "roadmap", fmap1.as_str());
        let mut grps = vec![];
        for brn in brns.iter() {
            println!("  MAIN: {brn}");
            let Some(chds) = pvassm.get(brn) else {
                println!("  ERROR #7 no child {brn}");
                continue;
            };
            let mut grp = vec![];
            for (i,chd) in chds.iter().enumerate() {
                println!("     {i}: {chd}");
                let Some(br) = cd_aoj.get(chd) else {
                    continue;
                };
                let mut off = vec![];
                for gon in br.gons.iter() {
                    let mut lns = vec![];
                    for (x,y) in gon.iter() {
                        lns.push((*x, *y));
                    }
                    off.push(lns);
                }
                grp.push(off);
            }
            grps.push(grp);
        }
        let fims = vec![fmap1,fmap2,fmap3,fmap4];
        draw_ass_map4(&map, po, &grps, fims);
    }
     Ok(())
}

pub fn draw_ass_map4(map: &GoogleMapInfo, pns: &Vec<Vec<(f64,f64)>>, grps: &Vec<Vec<Vec<Vec<(f32,f32)>>>>, fims: Vec<String>) {
    println!("MAP3================================");
    let fim1 = fims[0].as_str();
    let fim2 = fims[1].as_str();
    let fim3 = fims[2].as_str();
    let fim4 = fims[3].as_str();
        println!("CROP PROC #0");
    if let Ok(img) = ImageReader::open(fim1) && let Ok(mut img) = img.decode() {
        println!("CROP PROC #1");
        let (w, h) = (img.width(), img.height());
        let img = img.crop(map.mg, map.updw, w - map.mg, h - map.updw * 2);
        //let frdcp = format!("{}cp.jpeg", fpath);
        img.save(fim2).expect("?");

        let (w, h) = (img.width(), img.height());

        use std::fmt::Write;
        let mut jsmp = String::new();
        writeln!(jsmp, "{{ \"map\": [").expect("?");

        let (t_x, t_y) = (0, 0);
        let fcx = map.ww / map.ex_x;
        let fcy = map.ww / map.ex_x;
        let mut img0 = RgbaImage::new(w, h);

        let alp = 90;
        let colors = [
            Rgba([220, 0, 0, alp]),
            Rgba([0, 220, 0, alp]),
            Rgba([0, 0, 220, alp]),
            Rgba([220, 220, 0, alp]),
            Rgba([220, 0, 220, alp]),
            Rgba([0, 220, 220, alp]),
            Rgba([240, 0, 0, alp]),
            Rgba([0, 240, 0, alp]),
            Rgba([0, 0, 240, alp]),
            Rgba([240, 220, 0, alp]),
            Rgba([240, 0, 220, alp]),
            Rgba([0, 240, 220, alp]),
        ];

        let mut cn = 0;
        for grp in grps.iter() {
            for off in grp.iter() {
                for ln in off.iter() {
                    let mut v_pn = vec![];
                    let mut v_pf = vec![];
                    for (x,y) in ln.iter() {
                        let x = (*x - map.or_x) * fcx - map.ofs_x;
                        let y = (*y - map.or_y) * fcy;
                        let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
                        let pf = Point { x:ix as f32, y:iy as f32 };
                        let pn = Point { x:ix, y:iy };
                        if !v_pn.is_empty() && v_pn[v_pn.len()-1]==pn { continue; }
                        v_pn.push(pn);
                        v_pf.push(pf);
                    }
                    v_pn.pop();
                    v_pf.pop();
                    let ci = cn % colors.len();
                    //draw_polygon_mut(&mut img0, &v_pn, Rgba([0, 220, 220, 255]));
                    draw_polygon_mut(&mut img0, &v_pn, colors[ci]);
                    draw_hollow_polygon_mut(&mut img0, &v_pf, Rgba([0, 0, 0, 255]));
                }
            }
            cn += 1;
        }

        for po in pns.iter() {
            let mut v_pn = vec![];
            for pn in po.iter() {
                let x = (pn.0 as f32 - map.or_x) * fcx - map.ofs_x;
                let y = (pn.1 as f32 - map.or_y) * fcy;
                let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
                let pn = Point { x:ix as f32, y:iy as f32 };
                //let pn = Point { x:ix as i32, y:iy as i32 };
                if !v_pn.is_empty() && v_pn[v_pn.len()-1]==pn { continue; }
                v_pn.push(pn);
            }
            v_pn.pop();
            //draw_polygon_mut(&mut img0, &v_pn, Rgba([0, 220, 220, 255]));
            //draw_polygon_mut(&mut img0, &v_pn, Rgba([0, 220, 220, 255]));
            draw_hollow_polygon_mut(&mut img0, &v_pn, Rgba([180, 0, 0, 255]));
        }
        println!("fim3: {fim3}");

        img0.save(fim3).expect("?");

        let mut img1 = open(fim2).unwrap();
        let img2 = open(fim3).unwrap();
        img1.blend(&img2, pixel_mult, true, false).unwrap();
        img1.save(fim4).expect("?");
    }
}


pub fn draw_brn_task3(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let dnm = arif.ass.t(OUTDIR);
    let prv_map_dir = format!("{dnm}/map3");
    std::fs::create_dir_all(&prv_map_dir)?;

    // ============ BRANCH POLYGON BEGIN ==========
    let fnm = BRANCH_INFO2_FILE;
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((aojv, _)): Result<(Vec<BranchAoj>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("!!!! ERROR #7 Not decoded".into());
    };
    println!("============== AOJ {}", aojv.len());
    let mut cd_aoj = HashMap::<String,BranchAoj>::new();
    for aoj in aojv.into_iter() {
        cd_aoj.insert(aoj.code.clone(), aoj);
    }
    println!("============== AOJ HM {}", cd_aoj.len());
    // ============ BRANCH POLYGON BEGIN ==========

    // ============ SELECT 2 BEGIN ==========
    let fnm = format!("{dnm}/asssum-{:?}.bin", SumPrvBrn1);
    println!("TASK2 DNM:{fnm}");
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((assv0, _)): Result<(Vec<(PeaAssVar, Vec<usize>)>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    println!("assv: {}", assv0.len());
    // ============ SELECT 2 BEGIN ==========

    // ============ PROV BRANCH BEGIN ==========
    let fnm = format!("{dnm}/asssum-{:?}-m.bin", SumPrvBrn1);
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((pvassm, _)): Result<(HashMap<String,Vec<String>>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    /*
    for (k,v) in pvassm.iter() {
        println!("====== {k} {}", v.len());
    }
    */
    // ============ PROV BRANCH END ==========

    // ============ PROV BRANCH MAIN BEGIN ==========
    let fnm = format!("{dnm}/asssum-{:?}-b.bin", SumPrvBrn1);
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((brn_mb, _)): Result<(HashMap<String,Vec<String>>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    println!("====== BRN MB: {}", brn_mb.len());

    // ============ PROV BRANCH MAIN END ==========
    /*
    println!("pvassm: {}", pvassm.len());
    for (k,v) in pvassm.iter() {
        println!("BRN MB: {k} {v:?}");
    }
    */
    for pvas in assv0.iter() {
        let Some(mb) = pvassm.get(&pvas.0.pvid) else {
            continue;
        };
        println!("{} - mb:{} tr:{}", pvas.0.pvid, mb.len(), pvas.0.v[VarType::NoPeaTr as usize].v);
    }
    // ============ PROV BRANCH BEGIN ==========

    let gis_out = "/mnt/e/CHMBACK/pea-data/GISDB";
    let gis_dir = "/mnt/e/CHMBACK/pea-data/GIS/GIS-2024-09-12";
    let ly = "LB_Changwat";
        //let rg = "LB_AOJ";

    println!("read gis1 {gis_dir}");
    let ars = ar_list();
    let mut v_pvdb = vec![];
    let mut v_pvpo = vec![];
    for r in &ars {
        std::fs::create_dir_all(gis_out).expect("ERR");
        let rgf = format!("{gis_dir}/{r}/{ly}.dbf");
        println!("rgf {}", rgf);
        let mut reader = dbase::Reader::from_path_with_encoding(rgf.clone(),CP874).unwrap();
        for rc in reader.iter_records() {
            let rc = rc.unwrap();
            let r = db_rec(rc.clone());
            v_pvdb.push(r);
            //vdb.push(r);
        }
        let rgf = format!("{gis_dir}/{r}/{ly}.shp");
        if let Ok(mut reader) = shapefile::Reader::from_path(rgf.clone()) {
            for (gon, _rc) in reader.iter_shapes_and_records_as::<shapefile::Polygon, dbase::Record>() .flatten() {
                let mut ringpnts = Vec::<Vec<(f64, f64)>>::new();
                for ring in gon.into_inner() {
                    let mut pnts = Vec::<(f64, f64)>::new();
                    for pnt in ring.into_inner() {
                        pnts.push((pnt.x, pnt.y));
                    }
                    ringpnts.push(pnts);
                }
                v_pvpo.push(ringpnts);
            }
        }
    }
    println!("db: {} {}", v_pvdb.len(), v_pvpo.len());

    let mut pv_po_db = HashMap::<String,_>::new();
    for (po,db) in v_pvpo.iter().zip(v_pvdb.iter()) {
        let Some(DbfVal::Character(Some(nm))) = db.get("CHANGWAT_N") else {
            println!("NO FIELD CHANGWAT_N");
            continue;
        };
        //println!("PV: {nm}");
        pv_po_db.insert(nm.clone(), (po.clone(), db.clone()));
    }
    println!("pv_po_db: {}", pv_po_db.len());

    for (ass,_) in assv0.iter() {
        let Some((po,_db)) = pv_po_db.get(&ass.pvid) else {
            println!("!!!!! ERROR  {}", ass.pvid);
            continue;
        };
        //println!(">>>>>>>>>>>>  {}", ass.pvid);
        if !PROV_SET1.contains(&ass.pvid.as_str()) {
            continue;
        }

        let mg = MP_MG;
        let updw = MP_UPDW;
        let ww = MP_WW;
        let hh = MP_HH;

        let (w, _h) = (mg as f32 + ww, updw as f32 * 2.0 + hh);
        //pub gons: Vec<Vec<(f32, f32)>>,

        let mut pns = vec![];
        for po in po.iter() {
            for (x, y) in po.iter() {
                pns.push((*x as f32, *y as f32));
            }
        }
        if pns.is_empty() {
            println!("!!!! BRANCH {} is empty polygon", ass.pvid);
            continue;
        }

        println!("ALL BRANCH {}", pvassm.len());
        let Some(brns) = brn_mb.get(&ass.pvid) else {
            println!("ERROR #6 PROVINCE NOT FOUND {}", ass.pvid);
            continue;
        };
        println!(" BRANCH {}", brns.len());

        println!("PRV: {}", ass.pvid);
        println!("BRANCH OK: {} - {}", ass.pvid, pns.len());
        let fst = (pns[0].0, pns[1].1);
        let (mut x0, mut y0, mut x1, mut y1) = (fst.0, fst.1, fst.0, fst.1);
        for pnt in pns.iter() {
            x0 = x0.min(pnt.0);
            y0 = y0.min(pnt.1);
            x1 = x1.max(pnt.0);
            y1 = y1.max(pnt.1);
        }
        let (ox, oy) = ((x1 + x0) * 0.5f32, (y1 + y0) * 0.5f32);
        let wd = x1 - x0;
        let (o_ld, _o_ln) = utm_latlong(ox, oy);
        let zm = meter_pixel_to_zoom_lat_2(wd, ww as u32, o_ld);
        let mtpx = zoom_to_meter_pixel_lat(zm, o_ld);
        let ex_x = mtpx * ww;
        let ex_y = mtpx * hh;
        let (o_x, o_y) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        let (xx, yy) = utm_latlong(o_x, o_y);
        let or_x = o_x - ex_x / 2.0;
        let or_y = o_y - ex_y / 2.0;
        let ofs_x = 40f32;
        let map = GoogleMapInfo {
            mg,
            updw,
            ww,
            hh,
            ex_x,
            ex_y,
            or_x,
            or_y,
            ofs_x,
            w,
            xx,
            yy,
            zm,
        };
        let nm = ass.pvid.clone();
        let fmap1 = format!("{prv_map_dir}/{}-m1.jpeg", nm);
        let fmap2 = format!("{prv_map_dir}/{}-m2.jpeg", nm);
        let fmap3 = format!("{prv_map_dir}/{}-m3.png", nm);
        let fmap4 = format!("{prv_map_dir}/{}-m4.jpeg", nm);
        load_ggmap(&map, "roadmap", fmap1.as_str());
        let mut grps = vec![];
        for brn in brns.iter() {
            println!("  MAIN: {brn}");
            let Some(chds) = pvassm.get(brn) else {
                println!("  ERROR #7 no child {brn}");
                continue;
            };
            let mut grp = vec![];
            for (i,chd) in chds.iter().enumerate() {
                println!("     {i}: {chd}");
                let Some(br) = cd_aoj.get(chd) else {
                    continue;
                };
                let mut off = vec![];
                for gon in br.gons.iter() {
                    let mut lns = vec![];
                    for (x,y) in gon.iter() {
                        lns.push((*x, *y));
                    }
                    off.push(lns);
                }
                grp.push(off);
            }
            grps.push(grp);
        }
        let fims = vec![fmap1,fmap2,fmap3,fmap4];
        draw_ass_map3(&map, po, &grps, fims);
    }
    /*
*/

    /*
    for (po,db) in v_pvpo.iter().zip(v_pvdb.iter()) {

        let Some(DbfVal::Character(Some(nm))) = db.get("CHANGWAT_N") else {
            println!("NO FIELD CHANGWAT_N");
            continue;
        };
        let mg = MP_MG;
        let updw = MP_UPDW;
        let ww = MP_WW;
        let hh = MP_HH;

        let (w, _h) = (mg as f32 + ww, updw as f32 * 2.0 + hh);
        //pub gons: Vec<Vec<(f32, f32)>>,

        let mut pns = vec![];
        for po in po.iter() {
            for (x, y) in po.iter() {
                pns.push((*x as f32, *y as f32));
            }
        }
        pns.pop();
        if pns.is_empty() {
            println!("!!!! BRANCH {} is empty polygon", nm);
            continue;
        }
        println!("BRANCH OK: {nm} - {}", pns.len());
        let fst = (pns[0].0 as f32, pns[1].1 as f32);
        let (mut x0, mut y0, mut x1, mut y1) = (fst.0, fst.1, fst.0, fst.1);
        for pnt in pns.iter() {
            x0 = x0.min(pnt.0 as f32);
            y0 = y0.min(pnt.1 as f32);
            x1 = x1.max(pnt.0 as f32);
            y1 = y1.max(pnt.1 as f32);
        }
        let (ox, oy) = ((x1 + x0) * 0.5f32, (y1 + y0) * 0.5f32);
        let wd = x1 - x0;
        let (o_ld, _o_ln) = utm_latlong(ox, oy);
        let zm = meter_pixel_to_zoom_lat_2(wd, ww as u32, o_ld);
        let mtpx = zoom_to_meter_pixel_lat(zm, o_ld);
        let ex_x = mtpx * ww;
        let ex_y = mtpx * hh;
        let (o_x, o_y) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        let (xx, yy) = utm_latlong(o_x, o_y);
        let or_x = o_x - ex_x / 2.0;
        let or_y = o_y - ex_y / 2.0;
        let ofs_x = 40f32;
        let map = GoogleMapInfo {
            mg,
            updw,
            ww,
            hh,
            ex_x,
            ex_y,
            or_x,
            or_y,
            ofs_x,
            w,
            xx,
            yy,
            zm,
        };
        let fmap1 = format!("{prv_map_dir}/{}-m1.jpeg", nm);
        let fmap2 = format!("{prv_map_dir}/{}-m2.jpeg", nm);
        let fmap3 = format!("{prv_map_dir}/{}-m3.png", nm);
        let fmap4 = format!("{prv_map_dir}/{}-m4.jpeg", nm);
        load_ggmap(&map, "roadmap", fmap1.as_str());
        let fims = vec![fmap1,fmap2,fmap3,fmap4];
        draw_ass_map2(&map, po, fims);
    }
    */
     Ok(())
}

pub fn draw_brn_task2(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let dnm = arif.ass.t(OUTDIR);

    let fnm = format!("{dnm}/asssum-{:?}.bin", SumPrvBrn1);
    println!("TASK2 DNM:{fnm}");
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((assv0, _)): Result<(Vec<(PeaAssVar, Vec<usize>)>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    println!("assv: {}", assv0.len());

    let fnm = format!("{dnm}/asssum-{:?}-m.bin", SumPrvBrn1);
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("Not found {fnm}").into());
    };
    let Ok((pvassm, _)): Result<(HashMap<String,Vec<String>>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };
    println!("pvassm: {}", pvassm.len());
    for (k,v) in pvassm.iter() {
        println!("BRN MB: {k} {v:?}");
    }
    for pvas in assv0.iter() {
        let Some(mb) = pvassm.get(&pvas.0.pvid) else {
            continue;
        };
        println!("{} - mb:{} tr:{}", pvas.0.pvid, mb.len(), pvas.0.v[VarType::NoPeaTr as usize].v);
    }
    Ok(())
}

pub fn draw_brn_task1(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let dnm = arif.ass.t(OUTDIR);
    println!("DNM:{dnm}");
    let brn_map_dir = format!("{dnm}/map1");
    std::fs::create_dir_all(&brn_map_dir)?;

    let mut aojcds = HashSet::<String>::new();
    let mut aojs = vec![];
    let mut eg = ProcEngine::default();
    let mut cd_aoj = HashMap::<String, GisAoj>::new();

    for ar in ar_list() {
        eg.aojs(ar);
        for ao in eg.aojs.iter() {
            let ao1 = ao.clone();
            if let (Some(code), Some(name)) = (&ao.code, &ao.name) {
                let code = code.trim().to_string();
                aojcds.insert(code.clone());
                let code = code.to_string();
                let name = name.to_string();
                cd_aoj.insert(code.clone(), ao1);
                aojs.push((code, name));
            }
        }
    }
    println!("ALL AOJ 001 {} = {}", aojs.len(), aojcds.len());

    //let fnm = "/mnt/e/CHMBACK/pea-data/pea2/บัญชี 1 และ 5.xlsx";
    let fnm = "/mnt/e/CHMBACK/pea-data/pea2/pea-brn-list.xlsx";
    let xls = load_xlsx(&vec![fnm])?;
    println!("load XLSX");
    let l1 = BRANCH_PREF_PRV.len();
    let l2 = BRANCH_PREF_BRN.len();
    let mut v_brn = vec![];
    if let Some(sh) = xls.into_iter().next() {
        println!("========= {}", sh.name);
        for rw in sh.data {
            let mut rw0 = rw[0].to_string();
            rw0 = rw0.trim().to_string();
            if rw0.is_empty() || rw[1].is_empty() || rw[2].is_empty() {
                println!("EXCLUDE #3: {rw0}");
                continue;
            }
            //let mut prf = "".to_string();
            let mut prf: String;
            if let Some(n) = rw0.find(" ") {
                prf = rw0[0..n].to_string();
                if prf.ends_with(".") {
                    let ll = prf.len();
                    prf = prf[..ll - 1].to_string();
                }
                let n = n + 1;
                rw0 = rw0[n..].to_string();
            } else {
                println!("EXCLUDE #2: {rw0}");
                continue;
            }
            let pb: &str;
            if rw0.starts_with(BRANCH_PREF_PRV) {
                rw0 = rw0[l1..].to_string();
                pb = "P";
            } else if rw0.starts_with(BRANCH_PREF_BRN) {
                rw0 = rw0[l2..].to_string();
                pb = "B";
            } else {
                println!("EXCLUDE #1: {rw0}");
                continue;
            }
            let nm = rw0.to_string();
            let sz = rw[1].to_string();
            let iv = rw[2].to_string();
            v_brn.push((prf, pb, nm, sz, iv));
            //println!("'{prf}' '{pb}' '{nm}' S:'{sz}' I:'{iv}'");
        }
    } else {
        println!(">>>>>>>>>>>>>> {fnm}");
    }
    let mut v_branch = Vec::<BranchAoj>::new();
    for (pr, tp, nm, sz, iv) in v_brn.iter() {
        let wds = pr.split(".");
        let mut v_pr = vec![];
        for p in wds {
            v_pr.push(p.to_string());
        }
        let pr2 = v_pr.join(".");
        let mut v_pv = v_pr.clone();
        let is_prv = *tp == "P";
        v_pv.pop();
        let up = v_pv.join(".");
        let has_stock = iv == "มี";
        let name = nm.trim().to_string();
        let ii = v_branch.len();
        //println!("{pr}[{}]={pr2} -> {pv2} {tp} {nm} {sz} {iv}", v_pr.len());
        let brn = BranchAoj {
            ii,
            no: pr2.clone(),
            up,
            name,
            size: sz.to_string(),
            has_stock,
            is_prv,
            ..Default::default()
        };
        v_branch.push(brn);
    }
    println!("========  BRANCH : {}", v_branch.len());
    for i in 0..v_branch.len() {
        if i > 0 {
            for j in (0..=i).rev() {
                if v_branch[i].up == v_branch[j].no {
                    v_branch[i].pai = Some(j);
                    v_branch[j].chd.push(i);
                    break;
                }
            }
        }
    }
    for i in 0..v_branch.len() {
        let mut iv = vec![i];
        while let Some(ii) = iv.pop() {
            if v_branch[ii].has_stock {
                v_branch[i].i_stock = Some(ii);
                v_branch[ii].stock_for.push(i);
                break;
            }
            if let Some(pai) = v_branch[ii].pai {
                iv.push(pai);
            }
        }
    }
    for i in 0..v_branch.len() {
        let mut iv = vec![i];
        while let Some(ii) = iv.pop() {
            if v_branch[ii].is_prv {
                v_branch[i].i_prov = Some(ii);
                v_branch[ii].prov_memb.push(i);
                break;
            }
            if let Some(pai) = v_branch[ii].pai {
                iv.push(pai);
            }
        }
    }
    let mut aoj0 = aojs;
    aoj0.sort_by(|a, b| a.1.cmp(&b.1));

    println!("========>>>> BRANCH : {}", aoj0.len());

    let mut aojm1 = HashMap::<String, String>::new();
    let mut aojv1 = Vec::<(String, String)>::new();
    for (cd, nm) in aoj0.iter() {
        let mut nm4 = nm.to_string();
        if nm4.starts_with("กฟจ.") {
            let nm2 = nm4.chars().collect::<Vec<_>>();
            let nm3 = &nm2[4..];
            nm4 = nm3.iter().cloned().collect::<String>();
        }
        if nm4.starts_with("กฟส.") {
            let nm2 = nm4.chars().collect::<Vec<_>>();
            let nm3 = &nm2[4..];
            nm4 = nm3.iter().cloned().collect::<String>();
        }
        if nm4.starts_with("กฟย.") {
            let nm2 = nm4.chars().collect::<Vec<_>>();
            let nm3 = &nm2[4..];
            nm4 = nm3.iter().cloned().collect::<String>();
        }
        if nm4.starts_with("กฟอ.") {
            let nm2 = nm4.chars().collect::<Vec<_>>();
            let nm3 = &nm2[4..];
            nm4 = nm3.iter().cloned().collect::<String>();
        }
        if nm4.starts_with("กฟฟ.") {
            let nm2 = nm4.chars().collect::<Vec<_>>();
            let nm3 = &nm2[4..];
            nm4 = nm3.iter().cloned().collect::<String>();
        }
        if nm4.starts_with("เมือง") {
            let nm2 = nm4.chars().collect::<Vec<_>>();
            let nm3 = &nm2[5..];
            nm4 = nm3.iter().cloned().collect::<String>();
        }
        if nm4.starts_with("อ.") {
            let nm2 = nm4.chars().collect::<Vec<_>>();
            let nm3 = &nm2[2..];
            nm4 = nm3.iter().cloned().collect::<String>();
        }
        if nm4.starts_with("ต.") {
            let nm2 = nm4.chars().collect::<Vec<_>>();
            let nm3 = &nm2[2..];
            nm4 = nm3.iter().cloned().collect::<String>();
        }
        if nm4.starts_with("บ.") {
            let nm2 = nm4.chars().collect::<Vec<_>>();
            let nm3 = &nm2[2..];
            nm4 = nm3.iter().cloned().collect::<String>();
        }
        nm4 = nm4.replace(" (L)", "");
        nm4 = nm4.replace(" (M)", "");
        nm4 = nm4.replace(" (S)", "");
        nm4 = nm4.replace(" (XS)", "");
        nm4 = nm4.trim().to_string();

        if nm4 == "จตุรพักตร์พิมาน" {
            nm4 = "จตุรพักตรพิมาน".to_string();
        }
        if nm4 == "นิคมสร้างตนเองอ่าวน้อย" {
            nm4 = "นิคมสร้างตนเองตำบลอ่าวน้อย".to_string();
        }
        if nm4 == "ปทุมรัตน์" {
            nm4 = "ปทุมรัตต์".to_string();
        }
        if nm4 == "พิบูลย์รักษ" {
            nm4 = "พิบูลย์รักษ์".to_string();
        }
        if nm4 == "สหัสขันธุ์" {
            nm4 = "สหัสขันธ์".to_string();
        }
        if nm4 == "บ้านเกาะมุกด์" {
            nm4 = "เกาะมุกด์".to_string();
        }
        if nm4 == "เทอดไทย" {
            nm4 = "บ้านเทอดไทย".to_string();
        }
        if nm4 == "หนองเบน" {
            nm4 = "บ้านหนองเบน".to_string();
        }
        if nm4 == "การุ้ง" {
            nm4 = "เมืองการุ้ง".to_string();
        }
        if nm4 == "ขอนแก่น 2" {
            nm4 = "เมืองขอนแก่น 2".to_string();
        }
        if nm4 == "นครราชสีมา2(หัวทะเล)" {
            nm4 = "เมืองนครราชสีมา 2 (หัวทะเล)".to_string();
        }
        if nm4 == "นครราชสีมา3(สุรนารี)" {
            nm4 = "เมืองนครราชสีมา 3 (สุรนารี)".to_string();
        }
        //if nm4=="ปทุมธานี 2" { nm4 = "เมืองปทุมธานี 2".to_string(); }
        if nm4 == "ปาน" {
            nm4 = "เมืองปาน".to_string();
        }
        if nm4 == "พัทยา" {
            nm4 = "เมืองพัทยา".to_string();
        }
        if nm4 == "ยาง" {
            nm4 = "เมืองยาง".to_string();
        }
        if nm4 == "สมุทรสาคร 2 (บ้านแพ้ว)" {
            nm4 = "เมืองสมุทรสาคร 2 (บ้านแพ้ว)".to_string();
        }
        if nm4 == "สรวง" {
            nm4 = "เมืองสรวง".to_string();
        }
        if nm4 == "อุดรธานี2" {
            nm4 = "เมืองอุดรธานี 2".to_string();
        }
        if nm4 == "เชียงใหม่2" {
            nm4 = "เมืองเชียงใหม่ 2".to_string();
        }
        if nm4 == "ปทุมธานี 2" {
            nm4 = "เมืองปทุมธานี 2".to_string();
        }
        if nm4 == "ทรัพย์ไพรวัลย์" {
            nm4 = "บ้านทรัพย์ไพรวัลย์".to_string();
        }
        if nm4 == "ทุ่งอ้ายโห้" {
            nm4 = "บ้านทุ่งอ้ายโห้".to_string();
        }

        aojv1.push((cd.clone(), nm4.clone()));
        aojm1.insert(cd.clone(), nm.clone());
    }
    aojv1.sort_by(|a, b| a.1.cmp(&b.1));

    use std::collections::HashSet;

    println!("=======================================");
    let mut cd_gis = HashSet::<String>::new();
    for (cd, _) in aojv1.iter() {
        let cd = cd.trim();
        if cd_gis.contains(cd) {
            println!(">>>>>>>>>>>> Already exist {cd}");
        } else {
            cd_gis.insert(cd.to_string());
        }
    }
    let mut cn = 0;
    for ck in AOJ_ERR1.iter() {
        let ck = ck.to_string().trim().to_string();
        if !cd_gis.contains(&ck) {
            cn += 1;
            println!("=============== KEY NOT FOUND 1:{cn} [{ck}] ");
        }
    }

    v_branch.sort_by(|a, b| a.name.cmp(&b.name));

    let mut ern = 0;
    for (i, (a, b)) in aojv1.iter().zip(v_branch.iter_mut()).enumerate() {
        if a.1 == b.name {
            b.code = a.0.clone();
            continue;
        }
        println!("{i}.ERR: {} === {}", a.1, b.name);
        ern += 1;
    }
    println!("============ ERROR COUNT {ern} =============");

    v_branch.sort_by(|a, b| a.ii.cmp(&b.ii));

    let mut h_cd = HashSet::<String>::new();
    for b in v_branch.iter() {
        h_cd.insert(b.code.clone());
    }

    let mut cn = 0;
    for ck in AOJ_ERR1.iter() {
        let ck = ck.to_string().trim().to_string();
        if !h_cd.contains(&ck) {
            cn += 1;
            println!("=============== KEY NOT FOUND 3: {cn} [{ck}] ");
        }
    }

    for br in v_branch.iter_mut() {
        if let Some(ao) = cd_aoj.get(&br.code) {
            br.ar = ao.ar.clone();
            br.xmin = ao.xmin;
            br.ymin = ao.ymin;
            br.xmax = ao.xmax;
            br.ymax = ao.ymax;
            br.level = ao.level;
            br.center_x = ao.center_x;
            br.center_y = ao.center_y;
            br.sht_name = ao.sht_name.clone();
            br.shp_len = ao.shp_len;
            br.office = ao.office.clone();
            br.parent1 = ao.parent1.clone();
            br.pea = ao.pea.clone();
            br.ar_cd = ao.ar_cd.clone();
            br.shp_area = ao.shp_area;
            br.prv_cd = ao.prv_cd.clone();
            br.aoj_sz = ao.aoj_sz.clone();
            br.reg = ao.reg.clone();
            br.gons = ao.gons.clone();
        } else {
            println!("============ NOT FOUND AOJCD: {}", br.code);
        }
    }
    println!("BRN:{} AOJ:{}", v_branch.len(), cd_aoj.len());

    if let Ok(bin) = bincode::encode_to_vec(&v_branch, bincode::config::standard()) {
        println!("FNM: {BRANCH_INFO2_FILE} bin:{}", bin.len());
        if std::fs::write(BRANCH_INFO2_FILE, bin).is_ok() {
            println!("WRITE {BRANCH_INFO2_FILE}");
        }
    }

    for br in v_branch.iter() {
        let mg = MP_MG;
        let updw = MP_UPDW;
        let ww = MP_WW;
        let hh = MP_HH;

        let (w, _h) = (mg as f32 + ww, updw as f32 * 2.0 + hh);
        //pub gons: Vec<Vec<(f32, f32)>>,

        let mut pns = vec![];
        let mut pn1 = vec![];
        for po in br.gons.iter() {
            let mut pn2 = vec![];
            for (x, y) in po.iter() {
                pns.push((*x, *y));
                pn2.push((*x as f64, *y as f64));
            }
            pn1.push(pn2);
        }
        if pns.is_empty() {
            println!("!!!! BRANCH {} is empty polygon", br.code);
            continue;
        }
        println!("BRANCH OK: {}", br.code);
        let fst = (pns[0].0, pns[1].1);
        let (mut x0, mut y0, mut x1, mut y1) = (fst.0, fst.1, fst.0, fst.1);
        for pnt in pns.iter() {
            x0 = x0.min(pnt.0);
            y0 = y0.min(pnt.1);
            x1 = x1.max(pnt.0);
            y1 = y1.max(pnt.1);
        }
        let (ox, oy) = ((x1 + x0) * 0.5f32, (y1 + y0) * 0.5f32);
        let wd = x1 - x0;
        let (o_ld, _o_ln) = utm_latlong(ox, oy);
        let zm = meter_pixel_to_zoom_lat_2(wd, ww as u32, o_ld);
        let mtpx = zoom_to_meter_pixel_lat(zm, o_ld);
        let ex_x = mtpx * ww;
        let ex_y = mtpx * hh;
        let (o_x, o_y) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        let (xx, yy) = utm_latlong(o_x, o_y);
        let or_x = o_x - ex_x / 2.0;
        let or_y = o_y - ex_y / 2.0;
        let ofs_x = 40f32;
        let map = GoogleMapInfo {
            mg,
            updw,
            ww,
            hh,
            ex_x,
            ex_y,
            or_x,
            or_y,
            ofs_x,
            w,
            xx,
            yy,
            zm,
        };
        let fmap1 = format!("{brn_map_dir}/{}-m1.jpeg", br.code);
        let fmap2 = format!("{brn_map_dir}/{}-m2.jpeg", br.code);
        let fmap3 = format!("{brn_map_dir}/{}-m3.png", br.code);
        let fmap4 = format!("{brn_map_dir}/{}-m4.jpeg", br.code);
        load_ggmap(&map, "roadmap", fmap1.as_str());
        let fims = vec![fmap1,fmap2,fmap3,fmap4];
        draw_ass_map(&map, pns, fims);
        //draw_ass_map3(&map, &pn1, fims);
    }
    Ok(())
}

//use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
//use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::types::Bounds;
use headless_chrome::Browser;
use std::{thread, time};

pub fn load_ggmap(gm: &GoogleMapInfo, basemap: &str, fpath: &str) {
    //let frdrw = format!("{fpath}");
    loop {
        //let frdrw = frdrw.as_str();
        if !std::path::Path::new(fpath).exists() {
            let bnd = Bounds::Normal {
                left: None,
                top: None,
                width: Some(gm.w.into()),
                height: Some(gm.w.into()),
            };
            let url = "https://www.google.com/maps/@?api=1&map_action=map";
            let url = format!(
                "{url}&center={},{}&zoom={}&basemap={basemap}",
                gm.xx, gm.yy, gm.zm,
            );
            println!("{url}");
            let browser = Browser::default().expect("browser");
            let tab = browser.new_tab().expect("new tab");
            if tab.navigate_to(&url).is_err() {
                println!("!!! fail to navigate to");
                continue;
            }
            if tab.set_bounds(bnd).is_err() {
                println!("!!! fail to set bound");
                continue;
            }
            if tab.wait_until_navigated().is_err() {
                println!("!!! fail to wait");
                continue;
            }

            let ten_millis = time::Duration::from_millis(2000);
            thread::sleep(ten_millis);
            let jpeg_data = tab
                .capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg, None, None, true)
                .expect("capture");
            std::fs::write(fpath, jpeg_data).expect("image file");
            //println!("img2 = {url} wrote {fpath}");
        } else {
            //println!("image 2 skipped {fpath}");
        }
        break;
    }
}

use image::open;
use image::ImageReader;
use image::Rgba;
use image::RgbaImage;
use image_blend::pixelops::pixel_mult;
use image_blend::DynamicChops;
//use imageproc::drawing::draw_filled_circle_mut;
//use imageproc::drawing::draw_line_segment_mut;
use imageproc::drawing::draw_hollow_polygon_mut;
use imageproc::drawing::draw_polygon_mut;
use imageproc::point::Point;

pub fn draw_ass_map3(map: &GoogleMapInfo, pns: &Vec<Vec<(f64,f64)>>, grps: &Vec<Vec<Vec<Vec<(f32,f32)>>>>, fims: Vec<String>) {
    println!("MAP3================================");
    let fim1 = fims[0].as_str();
    let fim2 = fims[1].as_str();
    let fim3 = fims[2].as_str();
    let fim4 = fims[3].as_str();
        println!("CROP PROC #0");
    if let Ok(img) = ImageReader::open(fim1) && let Ok(mut img) = img.decode() {
        println!("CROP PROC #1");
        let (w, h) = (img.width(), img.height());
        let img = img.crop(map.mg, map.updw, w - map.mg, h - map.updw * 2);
        //let frdcp = format!("{}cp.jpeg", fpath);
        img.save(fim2).expect("?");

        let (w, h) = (img.width(), img.height());

        use std::fmt::Write;
        let mut jsmp = String::new();
        writeln!(jsmp, "{{ \"map\": [").expect("?");

        let (t_x, t_y) = (0, 0);
        let fcx = map.ww / map.ex_x;
        let fcy = map.ww / map.ex_x;
        let mut img0 = RgbaImage::new(w, h);

        let alp = 50;
        let colors = [
            Rgba([220, 0, 0, alp]),
            Rgba([0, 220, 0, alp]),
            Rgba([0, 0, 220, alp]),
            Rgba([220, 220, 0, alp]),
            Rgba([220, 0, 220, alp]),
            Rgba([0, 220, 220, alp]),
            Rgba([240, 0, 0, alp]),
            Rgba([0, 240, 0, alp]),
            Rgba([0, 0, 240, alp]),
            Rgba([240, 220, 0, alp]),
            Rgba([240, 0, 220, alp]),
            Rgba([0, 240, 220, alp]),
        ];

        for po in pns.iter() {
            let mut v_pn = vec![];
            let mut v_pf = vec![];
            for (x,y) in po.iter() {
                let x = (*x as f32 - map.or_x) * fcx - map.ofs_x;
                let y = (*y as f32 - map.or_y) * fcy;
                let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
                let pf = Point { x:ix as f32, y:iy as f32 };
                let pn = Point { x:ix, y:iy };
                if !v_pn.is_empty() && v_pn[v_pn.len()-1]==pn { continue; }
                v_pn.push(pn);
                v_pf.push(pf);
            }
            v_pn.pop();
            v_pf.pop();
            //draw_polygon_mut(&mut img0, &v_pn, Rgba([180,180,180,50]));
            draw_polygon_mut(&mut img0, &v_pn, Rgba([40,40,40,100]));
            draw_hollow_polygon_mut(&mut img0, &v_pf, Rgba([0, 0, 0, 255]));
        }

        let mut cn = 0;
        for grp in grps.iter() {
            for off in grp.iter() {
                for ln in off.iter() {
                    let mut v_pn = vec![];
                    let mut v_pf = vec![];
                    for (x,y) in ln.iter() {
                        let x = (*x - map.or_x) * fcx - map.ofs_x;
                        let y = (*y - map.or_y) * fcy;
                        let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
                        let pf = Point { x:ix as f32, y:iy as f32 };
                        let pn = Point { x:ix, y:iy };
                        if !v_pn.is_empty() && v_pn[v_pn.len()-1]==pn { continue; }
                        v_pn.push(pn);
                        v_pf.push(pf);
                    }
                    v_pn.pop();
                    v_pf.pop();
                    let ci = cn % colors.len();
                    cn += 1;
                    //draw_polygon_mut(&mut img0, &v_pn, Rgba([0, 220, 220, 255]));
                    draw_polygon_mut(&mut img0, &v_pn, colors[ci]);
                    draw_hollow_polygon_mut(&mut img0, &v_pf, Rgba([0, 0, 0, 255]));
                }
            }
        }

        println!("fim3: {fim3}");

        img0.save(fim3).expect("?");

        let mut img1 = open(fim2).unwrap();
        let img2 = open(fim3).unwrap();
        img1.blend(&img2, pixel_mult, true, false).unwrap();
        img1.save(fim4).expect("?");
    }
}

pub fn draw_ass_map2(map: &GoogleMapInfo, pns: &Vec<Vec<(f64,f64)>>, fims: Vec<String>) {
    let fim1 = fims[0].as_str();
    let fim2 = fims[1].as_str();
    let fim3 = fims[2].as_str();
    let fim4 = fims[3].as_str();
        println!("CROP PROC #0");
    if !std::path::Path::new(&fim2).exists()
        && let Ok(img) = ImageReader::open(fim1)
        && let Ok(mut img) = img.decode()
    {
        println!("CROP PROC #1");
        let (w, h) = (img.width(), img.height());
        let img = img.crop(map.mg, map.updw, w - map.mg, h - map.updw * 2);
        //let frdcp = format!("{}cp.jpeg", fpath);
        img.save(fim2).expect("?");

        let (w, h) = (img.width(), img.height());

        use std::fmt::Write;
        let mut jsmp = String::new();
        writeln!(jsmp, "{{ \"map\": [").expect("?");

        let (t_x, t_y) = (0, 0);
        let fcx = map.ww / map.ex_x;
        let fcy = map.ww / map.ex_x;
        let mut img0 = RgbaImage::new(w, h);

        for po in pns.iter() {
            let mut v_pn = vec![];
            for pn in po.iter() {
                let x = (pn.0 as f32 - map.or_x) * fcx - map.ofs_x;
                let y = (pn.1 as f32 - map.or_y) * fcy;
                let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
                let pn = Point { x:ix, y:iy };
                if !v_pn.is_empty() && v_pn[v_pn.len()-1]==pn { continue; }
                v_pn.push(pn);
            }
            v_pn.pop();
            draw_polygon_mut(&mut img0, &v_pn, Rgba([0, 220, 220, 255]));
        }
        println!("fim3: {fim3}");

        img0.save(fim3).expect("?");

        let mut img1 = open(fim2).unwrap();
        let img2 = open(fim3).unwrap();
        img1.blend(&img2, pixel_mult, true, false).unwrap();
        img1.save(fim4).expect("?");
    }
}

pub fn draw_ass_map(map: &GoogleMapInfo, pns: Vec<(f32,f32)>, fims: Vec<String>) {
    let fim1 = fims[0].as_str();
    let fim2 = fims[1].as_str();
    let fim3 = fims[2].as_str();
    let fim4 = fims[3].as_str();
        println!("CROP PROC #0");
    //if !std::path::Path::new(&fim2).exists()
    if let Ok(img) = ImageReader::open(&fim1) && let Ok(mut img) = img.decode() {
        println!("CROP PROC #1");
        let (w, h) = (img.width(), img.height());
        let img = img.crop(map.mg, map.updw, w - map.mg, h - map.updw * 2);
        //let frdcp = format!("{}cp.jpeg", fpath);
        img.save(fim2).expect("?");

        let (w, h) = (img.width(), img.height());

        use std::fmt::Write;
        let mut jsmp = String::new();
        writeln!(jsmp, "{{ \"map\": [").expect("?");

        let (t_x, t_y) = (0, 0);
        let fcx = map.ww / map.ex_x;
        let fcy = map.ww / map.ex_x;
        let mut img0 = RgbaImage::new(w, h);

        /*
        for pnt in pns.iter() {
            let x = (pnt.0 - map.or_x) * fcx - map.ofs_x;
            let y = (pnt.1 - map.or_y) * fcy;
            let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
            draw_filled_circle_mut(&mut img0, (ix, iy), 8, Rgba([0, 180, 255, 255]));
        }
        */

        use imageproc::point::Point;
        let mut v_pn = vec![];
        for pnt in pns.iter() {
            let x = (pnt.0 - map.or_x) * fcx - map.ofs_x;
            let y = (pnt.1 - map.or_y) * fcy;
            let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
            let pn = Point { x:ix, y:iy };
            //let (fx, fy) = (ix as f32, iy as f32);
            //let pn = Point { x:fx, y:fy };
            v_pn.push(pn);
        }
        v_pn.pop();
        //v_pn.push(v_pn[0]);
        //draw_hollow_polygon_mut(&mut img0, &v_pn, Rgba([0, 180, 180, 255]));
        draw_polygon_mut(&mut img0, &v_pn, Rgba([0, 220, 220, 255]));
        println!("fim3: {fim3}");

        img0.save(fim3).expect("?");

        let mut img1 = open(fim2).unwrap();
        let img2 = open(fim3).unwrap();
        img1.blend(&img2, pixel_mult, true, false).unwrap();
        img1.save(fim4).expect("?");
        /*
        for a in map.assv.iter() {
            let pno = format!("หม้อแปลง: {}", a.peano);
            let pnt = a.n1d.n1d_2_utm();

            let (lat, lon) = a.n1d.n1d_2_latlon();
            let x = (pnt.0 - map.or_x) * fcx - map.ofs_x;
            let y = (pnt.1 - map.or_y) * fcy;
            let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
            let (fx, fy) = (ix as f32, iy as f32);
            draw_filled_circle_mut(&mut img0, (ix, iy), 8, Rgba([0, 180, 255, 255]));
            draw_line_segment_mut(
                &mut img0,
                (fx, fy - 10.0),
                (fx, fy + 10.0),
                Rgba([0, 180, 0, 255]),
            );
            draw_line_segment_mut(
                &mut img0,
                (fx - 10.0, fy),
                (fx + 10.0, fy),
                Rgba([0, 180, 0, 255]),
            );
            if fg {
                write!(jsmp, ", ").expect("?");
            }
            writeln!(
                jsmp,
                "{{ \"x\": {ix}, \"y\": {iy}, \"rad\": 10, \"name\": \"{pno}\", \"lat\": {lat}, \"lon\": {lon} }}"
            )
            .expect("?");
            fg = true;
        }

        let ll1 = map.sub.n1d_f.n1d_2_latlon();
        let ll2 = map.sub.n1d_s.n1d_2_latlon();
        let (lat, lon) = ll1;
        let (mdx, mdy) = (map.xx, map.yy);
        println!("SUB: {},{}  {},{}", ll1.0, ll1.1, ll2.0, ll2.1);

        println!("MID: {mdx},{mdy}");
        println!("t:{t_x},{t_y}  w:{w} h:{h} fac:{fcx},{fcy}");

        let pnt = map.sub.n1d_f.n1d_2_utm();
        let x = (pnt.0 - map.or_x) * fcx - map.ofs_x;
        let y = (pnt.1 - map.or_y) * fcy;
        let (ix, iy) = (x as i32 + t_x, h as i32 - y as i32 + t_y);
        let (fx, fy) = (ix as f32, iy as f32);
        draw_filled_circle_mut(&mut img0, (ix, iy), 20, Rgba([255, 150, 255, 255]));

        draw_line_segment_mut(
            &mut img0,
            (fx - 50.0, fy - 50.0),
            (fx + 50.0, fy + 50.0),
            Rgba([180, 180, 0, 255]),
        );
        draw_line_segment_mut(
            &mut img0,
            (fx - 50.0, fy + 50.0),
            (fx + 50.0, fy - 50.0),
            Rgba([180, 150, 0, 255]),
        );

        if fg {
            write!(jsmp, ", ").expect("?");
        }
        let sno = format!("สถานีไฟฟ้า: {}", map.sub.sbid);
        writeln!(
            jsmp,
            "{{ \"x\": {ix}, \"y\": {iy}, \"rad\": 20, \"name\": \"{sno}\", \"lat\": {lat}, \"lon\": {lon} }}"
        )
        .expect("?");

        let frd01 = format!("{}01.png", fpath);
        img0.save(&frd01).expect("?");

        let mut img1 = open(&frdcp).unwrap();
        let img2 = open(&frd01).unwrap();
        img1.blend(&img2, pixel_mult, true, false).unwrap();
        img1.save(frd02).expect("?");

        writeln!(jsmp, "]}}").expect("?");
        //println!("map: {jsmp}");
        //let v: Value = serde_json::from_str(jsmp.as_str()).expect("?");
        //println!("jsmp: {v:?}");
        let fmp02 = format!("{}02.json", fpath);
        std::fs::write(fmp02, jsmp).unwrap();
        */
    }
}

pub fn read_aoj1() -> Result<(), Box<dyn Error>> {
    let gis_dir = "/mnt/e/CHMBACK/pea-data/GIS/GIS-2024-09-12";
    let gis_out = "/mnt/e/CHMBACK/pea-data/GISDB";
    println!("read gis1 {gis_dir}");
    let ars = ar_list();
    for r in &ars {
        std::fs::create_dir_all(gis_out).expect("ERR");
        //let rg = "LB_AOJ_Merge_Polygon";
        let rg = "LB_AOJ";
        let rgf = format!("{gis_dir}/{r}/{rg}.dbf");
        let mut vdb = vec![];
        println!("rgf {}", rgf);
        let mut reader = dbase::Reader::from_path_with_encoding(rgf.clone(),CP874).unwrap();
        for rc in reader.iter_records() {
            let rc = rc.unwrap();
            let r = db_rec(rc.clone());
            vdb.push(r);
        }
        let dbw = format!("{}/{}_{}.db", gis_out, r, rg);
        println!("{} f:{dbw}", vdb.len());
        if let Ok(bin) = bincode::encode_to_vec(&vdb, bincode::config::standard()) && let Err(e) = std::fs::write(&dbw, bin) {
            println!("ERROR write {dbw} {e:?}");
        }
    }
    Ok(())
}


