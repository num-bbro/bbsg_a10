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
    println!("===============  READ DATA FILE {fnm}");
    // ==== read rw3 data
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

pub fn sum_proc_e(coreno: usize, vnm: &str) -> Result<(), Box<dyn Error>> {
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
    let yrst = assu.v(PRJ_START_YEAR) as usize;
    let yrst = 2021;
    let yred = assu.v(PRJ_END_YEAR) as usize;
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
