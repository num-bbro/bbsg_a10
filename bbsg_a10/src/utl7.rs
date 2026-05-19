use bincode::Decode;
use bincode::Encode;
use sglab02_lib::sg::gis1::ar_list;
use shapefile::dbase;
use std::collections::HashMap;
use std::error::Error;
use rust_xlsxwriter::Workbook;
use crate::dcl::Pan;
//use rust_xlsxwriter::{Workbook,Format,FormatAlign,FormatBorder,Color};

// GIS data extraction
//

#[allow(non_camel_case_types)]
#[derive(Encode, Decode, Debug, Clone)]
pub enum DbfVal {
    Character(Option<String>),
    Numeric(Option<f64>),
    Logical(Option<bool>),
    Float(Option<f32>),
    Integer(i32),
    Currency(f64),
    Double(f64),
    Memo(String),
    None,
}

pub fn db_rec(rc: dbase::Record) -> HashMap<String, DbfVal> {
    let mut rec = HashMap::new();
    for (nm, va) in rc {
        let v = match &va {
            dbase::FieldValue::Character(op) => DbfVal::Character(op.clone()),
            dbase::FieldValue::Numeric(op) => DbfVal::Numeric(*op),
            dbase::FieldValue::Logical(op) => DbfVal::Logical(*op),
            dbase::FieldValue::Float(op) => DbfVal::Float(*op),
            dbase::FieldValue::Integer(i) => DbfVal::Integer(*i),
            dbase::FieldValue::Currency(c) => DbfVal::Currency(*c),
            dbase::FieldValue::Double(v) => DbfVal::Double(*v),
            dbase::FieldValue::Memo(s) => DbfVal::Memo(s.to_string()),
            _ => DbfVal::None,
        };
        rec.insert(nm, v);
    }
    rec
}

const POINT_LAYER: [&str;17] = [
    "DS_Capacitor",
    "DS_CircuitBreaker",
    "DS_Generator",
    "DS_HVCircuitbreaker",
    "DS_HVGenerator",
    "DS_HVPrimaryMeter",
    "DS_HVSwitch",
    "DS_HVTransformer",
    "DS_LowVoltageMeter",
    "DS_LVCapacitor",
    "DS_LVGenerator",
    "DS_PrimaryMeter",
    "DS_RECLOSER",
    "DS_Switch",
    "DS_SwitchingFacility",
    "DS_Transformer",
    "DS_VoltageRegulator",
];

const POLYGON_LAYER: [&str;5] = [
    "LB_Amphoe",
    "LB_AOJ",
    "LB_Changwat",
    "LB_Tambol",
    "Zone_Use",
];

const LINE_LAYER: [&str; 6] = [
    "DS_BusBar",
    "DS_EserviceLine",
    "DS_HVBusBar",
    "DS_HVConductor",
    "DS_LVConductor",
    "DS_MVConductor",
];

const DATA_LAYER: [&str; 3] = ["DS_GroupMeter_Detail", "GIS_HVMVCNL", "GIS_LVCNL"];

pub fn read_gis1() -> Result<(), Box<dyn Error>> {
    let gis_dir = "/mnt/e/CHMBACK/pea-data/GIS/GIS-2024-08-10";
    let gis_out = "/mnt/e/CHMBACK/pea-data/GISDB";
    println!("read gis1 {gis_dir}");
    let ars = ar_list();

    std::fs::create_dir_all(gis_out).expect("ERR");

    for r in ars {
        let agisdir = format!("{}/{}", gis_out, r);
        //let wdir = format!("../sgdata/db1");
        std::fs::create_dir_all(&agisdir).expect("ERR");

        // POLYGON
        for rg in POLYGON_LAYER {
            let rgf = format!("{gis_dir}/{r}/{rg}.shp");
            println!("rgf {}", rgf);
            let mut cnt = 0;
            let mut cnu = 0;
            let mut vrg = vec![];
            let mut vdb = vec![];
            if let Ok(mut reader) = shapefile::Reader::from_path(rgf.clone()) {
                for (gon, rc) in reader
                    .iter_shapes_and_records_as::<shapefile::Polygon, dbase::Record>()
                    .flatten()
                {
                    let mut ringpnts = Vec::<Vec<(f64, f64)>>::new();
                    for ring in gon.into_inner() {
                        let mut pnts = Vec::<(f64, f64)>::new();
                        for pnt in ring.into_inner() {
                            pnts.push((pnt.x, pnt.y));
                            //cnt += 1;
                        }
                        ringpnts.push(pnts);
                        cnt += 1;
                    }
                    cnu += 1;
                    vrg.push(ringpnts);
                    let r = db_rec(rc.clone());
                    vdb.push(r);
                }
            }
            println!("   rg: {} cnu:{} cnt:{}", rgf, cnu, cnt);
            let rgw = format!("{}/{}_{}.rg", agisdir, r, rg);
            if let Ok(bin) = bincode::encode_to_vec(&vrg, bincode::config::standard()) &&
               let Err(e) = std::fs::write(&rgw, bin) {
                        println!("write {rgw} {e:?}");
            }
            let dbw = format!("{}/{}_{}.db", agisdir, r, rg);
            if let Ok(bin) = bincode::encode_to_vec(&vdb, bincode::config::standard()) &&
                    let Err(e) = std::fs::write(&dbw, bin) {
                        println!("write {dbw} {e:?}");
            }
        }
        // POINT FILE
        for pn in POINT_LAYER {
            //let pnf = format!("{}/{}.shp", gis_dir, pn);
            let pnf = format!("{gis_dir}/{r}/{pn}.shp");
            println!("pnf {pnf}");
            let mut cnt = 0;
            let mut vpn = vec![];
            let mut vdb = vec![];
            if let Ok(mut reader) = shapefile::Reader::from_path(pnf.clone()) {
                for (pnt,rc) in reader.iter_shapes_and_records_as::<shapefile::Point, dbase::Record>().flatten() {
                    vpn.push((pnt.x, pnt.y));
                    let r = db_rec(rc);
                    vdb.push(r);
                    cnt += 1;
                }
            }
            println!("   pn {} {}", pnf, cnt);
            let pnw = format!("{}/{}_{}.pn", agisdir, r, pn);
            let dbw = format!("{}/{}_{}.db", agisdir, r, pn);
            if let Ok(bin) = bincode::encode_to_vec(&vpn, bincode::config::standard()) &&
            let Err(e) = std::fs::write(&pnw, bin) {
                println!("   write {pnw} {e:?}");
            }
            if let Ok(bin) = bincode::encode_to_vec(&vdb, bincode::config::standard()) &&
            let Err(e) = std::fs::write(&dbw, bin) {
                println!("   write {dbw} {e:?}");
            }
        }
    }
    Ok(())
}

pub fn read_gis2() -> Result<(), Box<dyn Error>> {
    let gis_dir = "/mnt/e/CHMBACK/pea-data/GIS/GIS-2024-08-10";
    let gis_out = "/mnt/e/CHMBACK/pea-data/GISDB";
    println!("read gis1 {gis_dir}");
    let ars = ar_list();

    std::fs::create_dir_all(gis_out).expect("ERR");

    for r in ars {
        let agisdir = format!("{}/{}", gis_out, r);
        std::fs::create_dir_all(&agisdir).expect("ERR");

        // LINE FILE
        for ln in LINE_LAYER {
            let lnf = format!("{gis_dir}/{r}/{ln}.shp");
            println!("rgf {}", lnf);
            let mut cnt = 0;
            let mut vdb = vec![];
            let mut vln = vec![];
            if let Ok(mut reader) = shapefile::Reader::from_path(&lnf) {
                for (line,rc) in
                    reader.iter_shapes_and_records_as::<shapefile::Polyline, dbase::Record>().flatten() 
                {
                        let mut lines = vec![];
                        for vpnts in line.into_inner() {
                            let mut line = vec![];
                            for pnt in vpnts {
                                line.push((pnt.x, pnt.y));
                            }
                            lines.push(line);
                        }
                        vln.push(lines);
                        let r = db_rec(rc);
                        vdb.push(r);
                        cnt += 1;
                }
            }
            println!("   ln: {lnf} cnt:{cnt}");
            let lnw = format!("{agisdir}/{r}_{ln}.ln");
            if let Ok(bin) = bincode::encode_to_vec(&vln, bincode::config::standard()) &&
               let Err(e) = std::fs::write(&lnw, bin) {
               println!("write {lnw} {e:?}");
            }
            let dbw = format!("{agisdir}/{r}_{ln}.db");
            if let Ok(bin) = bincode::encode_to_vec(&vdb, bincode::config::standard()) &&
               let Err(e) = std::fs::write(&dbw, bin) {
               println!("write {dbw} {e:?}");
            }
        }
        // DBASE FILE
        for db in DATA_LAYER {
            let dbf = format!("{agisdir}/{db}.dbf");
            let mut cnt = 0;
            let mut vdb = vec![];
            if let Ok(records) = dbase::read(dbf.clone()) {
                for rc in records {
                    let r = db_rec(rc);
                    vdb.push(r);
                    cnt += 1;
                }
            }
            println!("db: {dbf} {cnt}");
            let dbw = format!("{agisdir}/{r}_{db}.db");
            if let Ok(bin) = bincode::encode_to_vec(&vdb, bincode::config::standard()) &&
               let Err(e) = std::fs::write(&dbw, bin) {
               println!("write {dbw} {e:?}");
            }
        }

    }
    Ok(())
}

//const GIS_DIR: &str = "/mnt/e/CHMBACK/pea-data/GIS/GIS-2024-08-10";
const GIS_OUT: &str = "/mnt/e/CHMBACK/pea-data/GISDB";
//use crate::dcl::Pan;
use dbase::yore::code_pages::CP874;

pub fn read_aoj3() -> Result<Vec<(String,String)>, Box<dyn Error>> {

    let gis_out = "/mnt/e/CHMBACK/pea-data/GISDB";
    let ars = ar_list();
    let mut aojs = vec![];
    for r in &ars {
        let rg = "LB_AOJ";
        let dbw = format!("{}/{}_{}.db", gis_out, r, rg);
        //println!("rgf {}", dbw);
        let bin = std::fs::read(&dbw)?;
        //println!("{dbw} - len: {}", bin.len());
        let aoj: Result<(Vec<HashMap<String,DbfVal>>, usize),_> = bincode::decode_from_slice(&bin[..], bincode::config::standard());
        let Ok((aoj,_)) = aoj else {
            continue;
        };
        //println!("rec: {}", aoj.len());
        for rw in aoj {
            if let (Some(DbfVal::Character(Some(name))),Some(DbfVal::Character(Some(code)))) = (rw.get("NAME"), rw.get("CODE")) {
                aojs.push((code.to_string(), name.to_string()));
                //println!("{code:?} = {name:?}");
            }
        }
    }
    Ok(aojs)
}

pub fn read_aoj2() -> Result<(), Box<dyn Error>> {
    let gis_out = "/mnt/e/CHMBACK/pea-data/GISDB";
    let ars = ar_list();
    let mut aojs = vec![];
    for r in &ars {
        let rg = "LB_AOJ";
        let dbw = format!("{}/{}_{}.db", gis_out, r, rg);
        println!("rgf {}", dbw);
        let bin = std::fs::read(&dbw)?;
        println!("{dbw} - len: {}", bin.len());
        let aoj: Result<(Vec<HashMap<String,DbfVal>>, usize),_> = bincode::decode_from_slice(&bin[..], bincode::config::standard());
        let Ok((aoj,_)) = aoj else {
            continue;
        };
        println!("rec: {}", aoj.len());
        for rw in aoj {
            if let (Some(DbfVal::Character(Some(name))),Some(DbfVal::Character(Some(code)))) = (rw.get("NAME"), rw.get("CODE")) {
                aojs.push((code.to_string(), name.to_string()));
                println!("{code:?} = {name:?}");
            }
        }
    }
    let mut workbook = Workbook::new();
    let sht = workbook.add_worksheet();
    sht.set_name("AOJ")?;
    for (i,(cd,nm)) in aojs.iter().enumerate() {
        let l3 = &cd[4..];
        sht.write(i as u32, 0, cd)?;
        sht.write(i as u32, 1, nm)?;
        sht.write(i as u32, 3, l3.to_string())?;

    }
    workbook.save("aoj.xlsx")?;
     Ok(())
}

pub fn read_aoj0() -> Result<(), Box<dyn Error>> {
    let rgf = "/mnt/e/CHMBACK/pea-data/inp1/LB_AOJ_Merge_Polygon/LB_AOJ_Merge_Polygon.dbf";
    let mut vdb = vec![];
    println!("rgf {}", rgf);
    let mut reader = dbase::Reader::from_path_with_encoding(rgf,CP874).unwrap();
    for rc in reader.iter_records() {
        let rc = rc.unwrap();
        let r = db_rec(rc.clone());
        vdb.push(r);
    }
    Ok(())
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

    /*
    let ly = "LB_AOJ";
    for r in ars {
        let dbf = format!("{GIS_OUT}/{r}/{r}_{ly}.db");
        let bin = std::fs::read(&dbf)?;
        println!("{dbf} - {}", bin.len());
        let aoj: Result<(Vec<HashMap<String,DbfVal>>, usize),_> = bincode::decode_from_slice(&bin[..], bincode::config::standard());
        if let Ok((a,_)) = aoj  {
            if a.is_empty() { continue; }
            for d in a.iter() {
                println!("AOJ: {d:?}");
                break;
            }
            break;
        }
    }
    //for r in ars {
        //let wdir = format!("../sgdata/db1");
        //std::fs::create_dir_all(&agisdir).expect("ERR");
        // POLYGON
        //for rg in POLYGON_LAYER {
            //let rgf = format!("{gis_dir}/{r}/{rg}.shp");
            let mut cnt = 0;
            let mut cnu = 0;
            let mut vrg = vec![];
            let mut vdb = vec![];
            if let Ok(mut reader) = shapefile::Reader::from_path(rgf.clone()) {
                for (gon, rc) in reader
                    .iter_shapes_and_records_as::<shapefile::Polygon, dbase::Record>()
                    .flatten()
                {
                    let mut ringpnts = Vec::<Vec<(f64, f64)>>::new();
                    for ring in gon.into_inner() {
                        let mut pnts = Vec::<(f64, f64)>::new();
                        for pnt in ring.into_inner() {
                            pnts.push((pnt.x, pnt.y));
                            //cnt += 1;
                        }
                        ringpnts.push(pnts);
                        cnt += 1;
                    }
                    cnu += 1;
                    vrg.push(ringpnts);
                    let r = db_rec(rc.clone());
                    vdb.push(r);
                }
            }
            println!("   rg: {} cnu:{} cnt:{}", rgf, cnu, cnt);
            let rgw = format!("{}/{}_{}.rg", agisdir, r, rg);
            if let Ok(bin) = bincode::encode_to_vec(&vrg, bincode::config::standard()) &&
               let Err(e) = std::fs::write(&rgw, bin) {
                        println!("write {rgw} {e:?}");
            }
            let dbw = format!("{}/{}_{}.db", agisdir, r, rg);
            if let Ok(bin) = bincode::encode_to_vec(&vdb, bincode::config::standard()) &&
                    let Err(e) = std::fs::write(&dbw, bin) {
                        println!("write {dbw} {e:?}");
            }
    }
    Ok(())
}
            */


pub fn read_trans1() -> Result<(), Box<dyn Error>> {
    let ars = ar_list();
    let ly = "DS_Transformer";
    let mut cn = 0;
    let mut cp = 0;
    for r in ars {
        let dbf = format!("{GIS_OUT}/{r}/{r}_{ly}.db");
        let bin = std::fs::read(&dbf)?;
        println!("{dbf} - {}", bin.len());
        let trf: Result<(Vec<HashMap<String,DbfVal>>, usize),_> = bincode::decode_from_slice(&bin[..], bincode::config::standard());
        if let Ok((a,_)) = trf {
           cn += a.len();
           let mut pp = 0;
           for d in a.iter() {
               if let Some(ow) = d.get("OWNER") && let DbfVal::Character(Some(s)) = ow && s=="P" {
                   pp += 1;
               }
           }
           cp += pp;
           println!("  {}, {pp}", a.len());
        }
    }
    println!("all: {cn} {cp}");
    Ok(())
}

pub fn read_meter1() -> Result<(), Box<dyn Error>> {
    let ars = ar_list();
    let ly = "DS_LowVoltageMeter";
    let (mut pc0,mut cc0,mut me0,mut sp0,mut eg0, mut p00, mut p10,mut p20,mut p30,mut p40,mut p50,mut p60,mut p70,mut pf0) = (0,0,0,0,0,0,0,0,0,0,0,0,0,0);
    let mut nn = 0;
    for r in ars {
        let dbf = format!("{GIS_OUT}/{r}/{r}_{ly}.db");
        let bin = std::fs::read(&dbf)?;
        println!("{dbf} - {}", bin.len());
        let mets: Result<(Vec<HashMap<String,DbfVal>>, usize),_> = bincode::decode_from_slice(&bin[..], bincode::config::standard());
        let (mut pc,mut cc,mut me,mut sp,mut eg, mut p0, mut p1,mut p2,mut p3,mut p4,mut p5,mut p6,mut p7,mut pf) = (0,0,0,0,0,0,0,0,0,0,0,0,0,0);
        if let Ok((mets,_)) = mets {
            for d in mets.iter() {
                if let Some(ow) = d.get("OWNER") && let DbfVal::Character(Some(s)) = ow  {
                    match s.as_str() {
                        "PEA" => {pc += 1; }
                        "CUST" => {cc += 1; }
                        "MEA" => {me += 1; }
                        "SPP" => {sp += 1; }
                        "EGAT" => {eg += 1; }
                        a => { println!("OWNER: {a}"); break; }
                    }
                }
                if let Some(phs) = d.get("PHASEDESIG") && let DbfVal::Numeric(Some(n)) = phs  {
                    match n {
                        0.0 => { p0 += 1; }
                        1.0 => { p1 += 1; }
                        2.0 => { p2 += 1; }
                        3.0 => { p3 += 1; }
                        4.0 => { p4 += 1; }
                        5.0 => { p5 += 1; }
                        6.0 => { p6 += 1; }
                        7.0 => { p7 += 1; }
                        15.0 => { pf += 1; }
                        x => { println!("PHASE: {x}"); }
                    }
                }
            }
            nn += mets.len();
            println!("  {} P:{pc} {cc} g:{eg} m:{me} s:{sp} PHASE 0:{p0} A:{p1} B:{p2} C:{p3} 4:{p4} 5:{p5} 6:{p6} 7:{p7} 15:{pf}", mets.len());
            pc0 += pc;
            cc0 += cc;
            me0 += me;
            sp0 += sp;
            eg0 += eg;
            p00 += p0;
            p10 += p1;
            p20 += p2;
            p30 += p3;
            p40 += p4;
            p50 += p5;
            p60 += p6;
            p70 += p7;
            pf0 += pf;
        }
    }
    println!("  {nn} P:{pc0} C:{cc0} M:{me0} S:{sp0} G:{eg0} Phase 0:{p00} A:{p10} B:{p20} C:{p30} 4:{p40} 5:{p50} 6:{p60} 7:{p70} f:{pf0}");
    Ok(())
}

use crate::utl::load_xlsx;

pub const BRANCH_PREF_PRV: &str = "การไฟฟ้าส่วนภูมิภาคจังหวัด";
pub const BRANCH_PREF_BRN: &str = "การไฟฟ้าส่วนภูมิภาคสาขา";

pub const BRANCH_INFO_FILE: &str = "/mnt/e/CHMBACK/pea-data/pea2/branch_infos.bin";
pub type PeaBranchInfo = (Vec<PeaBranch>,HashMap<String,usize>);

#[derive(Debug, Encode, Decode, Default, Clone)]
pub struct PeaBranch {
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
}

pub const AOJ_ERR1: [&str;32] = [
"0901101",
"0719101",
"0717101",
"0720101",
"0706101",
"0910101",
"0914101",
"0918101",
"0706301",
"0901201",
"0914102",
"0726101",
"0824101",
"0314101",
"0825101",
"0723103",
"0516101",
"0417101",
"0711101",
"0824102",
"0314201",
"0314301",
"0824103",
"0314202",
"0613102",
"0417106",
"0417105",
"0417102",
"0417103",
"0417102",
"0208305",
"0825102",
];

use crate::dcl::ProcEngine;

pub fn read_branch1() -> Result<(), Box<dyn Error>> {
    //let err2 = ["0723103", "0711101", "0613102", "0208305"];

    let mut aojcds = HashSet::<String>::new();
    let mut aojs = vec![];
    let mut eg = ProcEngine::default();
    for ar in ar_list() {
        eg.aojs(ar);
        for ao in eg.aojs.iter() {
            if let (Some(code),Some(name)) = (&ao.code,&ao.name) {
                let code = code.trim().to_string();
                //if err2.contains(&code.as_str()) {
                //    println!(">>>>>>>>>>>>>>> {code} = {ao:?}");
                //}
                aojcds.insert(code.clone());
                //aojs.push(ao.clone());
                let code = code.to_string();
                let name = name.to_string();
                aojs.push((code,name));
            }
        }
    }
    println!("ALL AOJ 001 {} = {}", aojs.len(), aojcds.len());
    /*
    for ao in AOJ_ERR1.iter() {
        let ao = ao.to_string();
        //println!("!!!!!!!!!!!!!  ERROR is {ao} = {:?}", aojcds.get(&ao));
    }
    println!("===========================================");
    */

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
                    prf = prf[..ll-1].to_string();
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
    let mut v_branch = Vec::<PeaBranch>::new();
    for (pr,tp,nm,sz,iv) in v_brn.iter() {
        let wds = pr.split(".");
        let mut v_pr = vec![];
        for p in wds {
            v_pr.push(p.to_string());
        }
        let pr2 = v_pr.join(".");
        let mut v_pv = v_pr.clone();
        let is_prv = *tp=="P";
        v_pv.pop();
        let up = v_pv.join(".");
        let has_stock = iv=="มี";
        let name = nm.trim().to_string();
        let ii = v_branch.len();
        //println!("{pr}[{}]={pr2} -> {pv2} {tp} {nm} {sz} {iv}", v_pr.len());
        let brn = PeaBranch {
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
        if i>0 {
            for j in (0..=i).rev() {
                if v_branch[i].up==v_branch[j].no { 
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
    /*
    for b in v_branch.iter() {
        println!("{}", b.name);
    }
    //use std::collections::HashSet;
    let Ok(mut aoj0) = read_aoj3() else {
        return Err("Cannot read aoj file".into());
    };
    */
/*
    let rgf = "/mnt/e/CHMBACK/pea-data/inp1/LB_AOJ_Merge_Polygon/LB_AOJ_Merge_Polygon.dbf";
    let mut aoj0 = vec![];
    println!("rgf {}", rgf);
    let mut reader = dbase::Reader::from_path_with_encoding(rgf,CP874).unwrap();
    for rc in reader.iter_records() {
        let rc = rc.unwrap();
        let r = db_rec(rc.clone());
        if let (Some(DbfVal::Character(Some(name))),Some(DbfVal::Character(Some(code)))) = (r.get("NAME"), r.get("CODE")) {
            aoj0.push((code.to_string(), name.to_string()));
        }
    }
    for (i,(c,n)) in aojs.iter().enumerate() {
        println!("{i}.{c} {n}");
    }
*/

    let mut aoj0 = aojs;
    aoj0.sort_by(|a,b| a.1.cmp(&b.1));

    println!("========>>>> BRANCH : {}", aoj0.len());

    let mut aojm1 = HashMap::<String,String>::new();
    let mut aojv1 = Vec::<(String,String)>::new();
    for (cd,nm) in aoj0.iter() {
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
        nm4 = nm4.replace(" (L)","");
        nm4 = nm4.replace(" (M)","");
        nm4 = nm4.replace(" (S)","");
        nm4 = nm4.replace(" (XS)","");
        nm4 = nm4.trim().to_string();

        if nm4=="จตุรพักตร์พิมาน" { nm4 = "จตุรพักตรพิมาน".to_string(); }
        if nm4=="นิคมสร้างตนเองอ่าวน้อย" { nm4 = "นิคมสร้างตนเองตำบลอ่าวน้อย".to_string(); }
        if nm4=="ปทุมรัตน์" { nm4 = "ปทุมรัตต์".to_string(); }
        if nm4=="พิบูลย์รักษ" { nm4 = "พิบูลย์รักษ์".to_string(); }
        if nm4=="สหัสขันธุ์" { nm4 = "สหัสขันธ์".to_string(); }
        if nm4=="บ้านเกาะมุกด์" { nm4 = "เกาะมุกด์".to_string(); }
        if nm4=="เทอดไทย" { nm4 = "บ้านเทอดไทย".to_string(); }
        if nm4=="หนองเบน" { nm4 = "บ้านหนองเบน".to_string(); }
        if nm4=="การุ้ง" { nm4 = "เมืองการุ้ง".to_string(); }
        if nm4=="ขอนแก่น 2" { nm4 = "เมืองขอนแก่น 2".to_string(); }
        if nm4=="นครราชสีมา2(หัวทะเล)" { nm4 = "เมืองนครราชสีมา 2 (หัวทะเล)".to_string(); }
        if nm4=="นครราชสีมา3(สุรนารี)" { nm4 = "เมืองนครราชสีมา 3 (สุรนารี)".to_string(); }
        //if nm4=="ปทุมธานี 2" { nm4 = "เมืองปทุมธานี 2".to_string(); }
        if nm4=="ปาน" { nm4 = "เมืองปาน".to_string(); }
        if nm4=="พัทยา" { nm4 = "เมืองพัทยา".to_string(); }
        if nm4=="ยาง" { nm4 = "เมืองยาง".to_string(); }
        if nm4=="สมุทรสาคร 2 (บ้านแพ้ว)" { nm4 = "เมืองสมุทรสาคร 2 (บ้านแพ้ว)".to_string(); }
        if nm4=="สรวง" { nm4 = "เมืองสรวง".to_string(); }
        if nm4=="อุดรธานี2" { nm4 = "เมืองอุดรธานี 2".to_string(); }
        if nm4=="เชียงใหม่2" { nm4 = "เมืองเชียงใหม่ 2".to_string(); }
        if nm4=="ปทุมธานี 2" { nm4 = "เมืองปทุมธานี 2".to_string(); }
        if nm4=="ทรัพย์ไพรวัลย์" { nm4 = "บ้านทรัพย์ไพรวัลย์".to_string(); }
        if nm4=="ทุ่งอ้ายโห้" { nm4 = "บ้านทุ่งอ้ายโห้".to_string(); }

        aojv1.push((cd.clone(), nm4.clone()));
        aojm1.insert(cd.clone(), nm.clone());
    }
    aojv1.sort_by(|a,b| a.1.cmp(&b.1));

    use std::collections::HashSet;

    println!("=======================================");
    let mut cd_gis = HashSet::<String>::new();
    for (cd,_) in aojv1.iter() {
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

    /*
    let mut brnv2 = vec![];
    for bb in v_branch.iter() { brnv2.push(bb.name.clone()) }
    brnv2.sort();
    */
    v_branch.sort_by(|a,b| a.name.cmp(&b.name));

    let mut ern = 0;
    for (i,(a,b)) in aojv1.iter().zip(v_branch.iter_mut()).enumerate() {
        if a.1==b.name {
            b.code = a.0.clone();
            continue;
        }
        println!("{i}.ERR: {} === {}", a.1, b.name);
        ern += 1;
    }
    println!("============ ERROR COUNT {ern} =============");

    v_branch.sort_by(|a,b| a.ii.cmp(&b.ii));
    
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
    
    /*
    let mut brnv2 = v_branch.clone();
    brnv2.sort_by(|a,b| a.name.cmp(&b.name));
    println!("================ aoj:{} brn:{}", aojv1.len(), brnv2.len());

    for (i,(a,b)) in aojv1.iter().zip(brnv2.iter()).enumerate() {
        if a.1==*b { continue; }
        println!("{i}. G:'{}' = D:'{}'", a.1, b);
    }
    for a in aojv1.iter() {
        println!("{}",a.1);
    }

    */
    /*
    let mut top = vec![];
    let mut nm_brn = HashMap::<String,usize>::new();
    for (i,brn) in v_branch.iter().enumerate() {
        if brn.pai.is_none() { top.push(i); }
        nm_brn.insert(brn.name.clone(), i);
    }
    */
    /*
    for (cd,nm) in aojv1.iter() {
        if let Some(ii) = nm_brn.get(nm) {
            v_branch[*ii].org = nm.to_string();
            v_branch[*ii].code = cd.to_string();
        } else {
            println!("========= ERROR #1 {nm}");
        }
    }
    let mut b_cds = HashMap::<String,usize>::new();
    for b in v_branch.iter() {
        let ent = b_cds.entry(b.code.clone()).or_insert_with(|| {0});
        *ent += 1;
    }
    let mut cn = 0;
    for ck in AOJ_ERR1.iter() {
        let ck = ck.to_string();
        if !b_cds.contains_key(&ck) {
            cn += 1;
            println!("=============== KEY NOT FOUND 2:{cn} '{ck}' ");
        }
    }
    */
    if let Ok(bin) = bincode::encode_to_vec(&v_branch, bincode::config::standard()) {
        println!("FNM: {BRANCH_INFO_FILE} bin:{}",bin.len());
        if std::fs::write(BRANCH_INFO_FILE, bin).is_ok() {
            println!("WRITE {BRANCH_INFO_FILE}");
        }
    }
    Ok(())
}

//pub fn get_brn_map() -> Result<(Vec<PeaBranch>,HashMap<String,usize>), Box<dyn Error>> {
pub fn get_brn_map() -> Result<PeaBranchInfo, Box<dyn Error>> {
    let bin = std::fs::read(BRANCH_INFO_FILE)?;
    let Ok((v_aoj,_)): Result<(Vec<PeaBranch>, usize),_> = bincode::decode_from_slice(&bin[..], bincode::config::standard()) else {
        return Err("Can open file".into());
    };
    let mut aoj_i = HashMap::<String,usize>::new();
    for (i,ao) in v_aoj.iter().enumerate() {
        //println!("??? {i}. {}, {}", ao.code, ao.name);
        aoj_i.insert(ao.code.to_string(), i);
    }
    for ck in AOJ_ERR1.iter() {
        let cd = ck.to_string();
        if !aoj_i.contains_key(&cd) {
            println!("CODE '{cd}' NOT FOUND");
        }
    }
    Ok((v_aoj,aoj_i))
}

pub const SUB_PHYS_FILE: &str = "/mnt/e/CHMBACK/pea-data/pea2/sub_phys_file.bin";

use crate::p04::SubFeedTrans;
use crate::utl6::archi_xml_read0;
use crate::utl6::archi_analyze;
use crate::utl6::get_assum_in_view;
use crate::utl6::ARCHI_INPUT;
use crate::utl6::archi_extract0;
use crate::asm::ASM::*;

#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct EconCalcInfo {
    pub no: i32,
    pub pvid: String,
    pub iret: f32,
    pub capex: f32,
    pub opex: f32,
    pub intrs: f32,
    pub cost: f32,
    pub npv: f32,
    pub irr: f32,
    pub brkyr: f32,
}

use crate::sty3::AssSumEnum;
pub fn econ_calc_file(vwnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vwnm, &mut arif)?;
    let assu = arif.assumption();
    let path = assu.t(ECON_CALC_EXCEL_FILE);
    println!("vwnm: {vwnm} {path}");
    let xls = load_xlsx(&vec![&path])?;
    let mut sht = None;

    for sh in xls.into_iter() {
        if sh.shnm=="Process" {
            sht = Some(sh.clone());
        }
        println!("========= {}", sh.shnm);
    }
    let mut econs = vec![];
    let Some(sht) = sht else { return Err("Sheet Not Found".into()); };
    for (i,rw) in sht.data.iter().enumerate() {
        if i<3 { continue; }
        let no = rw[0].parse::<i32>().unwrap_or(0i32);
        let pvid = rw[1].to_string();
        let iret = rw[2].parse::<f32>().unwrap_or(0f32);
        let capex = rw[3].parse::<f32>().unwrap_or(0f32);
        let opex = rw[4].parse::<f32>().unwrap_or(0f32);
        let intrs = rw[5].parse::<f32>().unwrap_or(0f32);
        let cost = rw[6].parse::<f32>().unwrap_or(0f32);
        let npv = rw[7].parse::<f32>().unwrap_or(0f32);
        let irr = rw[8].parse::<f32>().unwrap_or(0f32);
        let brkyr = rw[9].parse::<f32>().unwrap_or(0f32);
        let econ = EconCalcInfo { no, pvid, iret, capex, opex, intrs, cost, npv, irr, brkyr, };
        if no>74 || no<=0 { break; }
        //println!("{econ:?}");
        econs.push(econ);
    }
    econs.sort_by(|a,b| b.npv.partial_cmp(&a.npv).unwrap());
    for (i,econ) in econs.iter().enumerate() {
        println!("{i}. {} npv:{} irr:{}", econ.pvid, econ.npv.pan(0), econ.irr.pan(2));
    }


    let dnm = assu.t(OUTDIR);
    let bin: Vec<u8> = bincode::encode_to_vec(&econs, bincode::config::standard()).unwrap();
    let fnm = format!("{dnm}/econ-calc-{:?}.bin", AssSumEnum::SumPrvBrn2);
    std::fs::write(&fnm, bin).unwrap();
    println!("file '{fnm}' saved");

    //pub data: Vec<Vec<String>>,
    /*
    let fnm = "/mnt/e/CHMBACK/pea-data/pea2/pea-brn-list.xlsx";
    println!("load XLSX");
    let l1 = BRANCH_PREF_PRV.len();
    let l2 = BRANCH_PREF_BRN.len();
    let mut v_brn = vec![];
    }
    */
    Ok(())
}

pub fn make_sub_phys() -> Result<(), Box<dyn Error>> {
    let mut subhm = HashMap::<String,SubFeedTrans>::new();
    for id in ar_list() {
        println!("make sub a:{id}");
        let mut eg = ProcEngine::default();
        eg.subs(id);
        for mut sub in eg.subs.into_iter() {
            sub.feed.clear();
            subhm.insert(sub.sbid.clone(), sub);
        }
    }
    println!("sub: {}", subhm.len());
    if let Ok(bin) = bincode::encode_to_vec(&subhm, bincode::config::standard()) {
        println!("FNM: {SUB_PHYS_FILE} bin:{}",bin.len());
        if std::fs::write(SUB_PHYS_FILE, bin).is_ok() {
            println!("WRITE {SUB_PHYS_FILE}");
        }
    }
    let subhm2 = get_sub_phys()?;
    println!("Sub Phys check: {}", subhm2.len());
    Ok(())
}

pub fn get_sub_phys() -> Result<HashMap<String,SubFeedTrans>, Box<dyn Error>> {
    let bin = std::fs::read(SUB_PHYS_FILE)?;
    let Ok((subhm,_)): Result<(HashMap<String,SubFeedTrans>, usize),_> = bincode::decode_from_slice(&bin[..], bincode::config::standard()) else {
        return Err("Can open file".into());
    };
    Ok(subhm)
}


/*
pub fn get_brn_info() -> Result<HashMap<String,PeaBranch>, Box<dyn Error>> {
    let bin = std::fs::read(BRANCH_INFO_FILE)?;
    let aoj: Result<(Vec<PeaBranch>, usize),_> = bincode::decode_from_slice(&bin[..], bincode::config::standard()) ;
    let mut brnif = HashMap::<String,PeaBranch>::new();
    if let Ok((aoj,_)) = aoj {
        for (i,ao) in aoj.into_iter().enumerate() {
            let cd = ao.code.clone();
            let nm = ao.name.clone();
            let sk = ao.has_stock;
            let sl = ao.stock_for.len();
            let is = ao.i_stock;
            //println!("{i}.{cd}-{nm} b:{sk} l:{sl} si:{is:?}");
            brnif.insert(ao.code.to_string(), ao.clone());
        }
    }
    Ok(brnif)
}
*/

/*
pub fn p12_read_aoj() -> Result<(), Box<dyn Error>> {
    let fdir = "/mnt/e/CHMBACK/pea-data/inp1";
    let ly = "LB_AOJ_Merge_Polygon";
    let frg = format!("{fdir}/gis/{ly}.rg");
    let fat = format!("{fdir}/gis/{ly}.at");
    println!("{frg} - {fat}");
    let mut a1 = HashMap::<String, usize>::new();
    let mut a2 = HashMap::<String, usize>::new();
    let mut a3 = HashMap::<String, usize>::new();
    if let (Ok(frg), Ok(fat)) = (File::open(&frg), File::open(&fat)) {
        let frg = BufReader::new(frg);
        let fat = BufReader::new(fat);
        if let (Ok(frg), Ok(fat)) = (
            bincode::deserialize_from::<BufReader<File>, Vec<Vec<Vec<(f64, f64)>>>>(frg),
            bincode::deserialize_from::<BufReader<File>, Vec<HashMap<String, DbfData>>>(fat),
        ) {
            let mut cn = 0;
            let mut ar_aojs = HashMap::<String, Vec<GisAoj>>::new();
            for (_i, (rg, at)) in frg.iter().zip(fat.iter()).enumerate() {
                cn += 1;

                let xmin = if let Some(DbfData::Real(v)) = at.get("XMIN") {
                    Some(*v as f32)
                } else {
                    None
                };
                let ymin = if let Some(DbfData::Real(v)) = at.get("YMIN") {
                    Some(*v as f32)
                } else {
                    None
                };
                let xmax = if let Some(DbfData::Real(v)) = at.get("XMAX") {
                    Some(*v as f32)
                } else {
                    None
                };
                let ymax = if let Some(DbfData::Real(v)) = at.get("YMAX") {
                    Some(*v as f32)
                } else {
                    None
                };
                //pub aoj_lv: f32, AOJ_LEVEL - Real(1.0)
                let level = if let Some(DbfData::Real(v)) = at.get("AOJ_LEVEL") {
                    Some(*v as f32)
                } else {
                    None
                };
                //pub center_x: f32, CENTROID_X - Real(11195412.0)
                let center_x = if let Some(DbfData::Real(v)) = at.get("CENTER_X") {
                    Some(*v as f32)
                } else {
                    None
                };
/mnt/e/CHMBACK/pea-data/inp1
                //pub center_y: f32, CENTROID_Y - Real(1599200.75)
                let center_y = if let Some(DbfData::Real(v)) = at.get("CENTER_Y") {
                    Some(*v as f32)
                } else {
                    None
                };
                //pub code: String, CODE - Text(0714101)
                let code = if let Some(DbfData::Text(s)) = at.get("CODE") {
                    Some(s.to_string())
                } else {
                    None
                };
                //pub sht_name: String, SHORT_NAME - Text(กฟส.)
                let sht_name = if let Some(DbfData::Text(s)) = at.get("SHORT_NAME") {
                    Some(s.to_string())
                } else {
                    None
                };
                //pub shp_len: f32, Shape_Leng - Real(96850.34375)
                let shp_len = if let Some(DbfData::Real(v)) = at.get("Shape_Leng") {
                    Some(*v as f32)
                } else {
                    None
                };
                //pub office: String, OFFICE - Text(GBIN)
                let office = if let Some(DbfData::Text(s)) = at.get("OFFICE") {
                    Some(s.to_string())
                } else {
                    None
                };
                //pub parent1: String, PARENT_1 - Text(G
                let parent1 = if let Some(DbfData::Text(s)) = at.get("PARENT_1") {
                    Some(s.to_string())
                } else {
                    None
                };
                //pub parent2: String, PARENT_2 - None
                let parent2 = if let Some(DbfData::Text(s)) = at.get("PARENT_2") {
                    Some(s.to_string())
                } else {
                    None
                };
                //pub pea: String, PEACODE - Text(G14101)
                let pea = if let Some(DbfData::Text(s)) = at.get("PEACODE") {
                    Some(s.to_string())
                } else {
                    None
                };
                //pub ar_cd: String, AREA_CODE - Text(31)
                let ar_cd = if let Some(DbfData::Text(s)) = at.get("AREA_CODE") {
                    Some(s.to_string())
                } else {
                    None
                };
                //pub shp_area: f32, Shape_Area - Real(198580080.0)
                let shp_area = if let Some(DbfData::Real(v)) = at.get("Shape_Area") {
                    Some(*v as f32)
                } else {
                    None
                };
                //pub prv_cd: String, PROV_CODE - Text(14)
                let prv_cd = if let Some(DbfData::Text(s)) = at.get("PROV_CODE") {
                    Some(s.to_string())
                } else {
                    None
                };
                //pub aoj_sz: String, AOJ_SIZE - Text(L)
                let aoj_sz = if let Some(DbfData::Text(s)) = at.get("AOJ_SIZE") {
                    Some(s.to_string())
                } else {
                    None
                };
                //pub reg: String, REGION - Text(G)
                let reg = if let Some(DbfData::Text(s)) = at.get("REGION") {
                    Some(s.to_string())
                } else {
                    None
                };
                //pub name: String, NAME - Text(กฟส)
                let name = if let Some(DbfData::Text(s)) = at.get("NAME") {
                    Some(s.to_string())
                } else {
                    None
                };
                let mut gons = Vec::<Vec<(f32, f32)>>::new();
                for rg1 in rg {
                    let mut gon = Vec::<(f32, f32)>::new();
                    for rg2 in rg1 {
                        gon.push((rg2.0 as f32, rg2.1 as f32));
                    }
                    gons.push(gon);
                }
                //==== area
                let off = office.clone().unwrap().to_string();
                let off = (&off[0..1]).to_string();
                if let Some(cn) = a1.get_mut(&off) {
                    *cn += 1;
                } else {
                    a1.insert(off.to_string(), 1);
                }
                let arcd = ar_cd.clone().unwrap().to_string();
                let arcd = (&arcd[0..2]).to_string();
                if let Some(cn) = a2.get_mut(&arcd) {
                    *cn += 1;
                } else {
                    a2.insert(arcd.to_string(), 1);
                }
                let peacd = pea.clone().unwrap().to_string();
                let peacd = (&peacd[0..1]).to_string();
                if let Some(cn) = a3.get_mut(&peacd) {
                    *cn += 1;
                } else {
                    a3.insert(peacd.to_string(), 1);
                }
                let ar1 = PEA_AR_CD.get(&arcd).unwrap_or(&"XX");
                let ar2 = PEA_AR_CD2.get(&peacd).unwrap_or(&"XX");
                let ar = if ar1 == ar2 {
                    ar1.to_string()
                } else if ar1 != &"XX" {
                    println!("ERROR1 {ar_cd:?}");
                    ar1.to_string()
                } else {
                    println!("ERROR2 {office:?}");
                    ar2.to_string()
                };
                let aoj = GisAoj {
                    ar,
                    xmin,
                    ymin,
                    xmax,
                    ymax,
                    level,
                    center_x,
                    center_y,
                    code,
                    sht_name,
                    shp_len,
                    office,
                    parent1,
                    parent2,
                    pea,
                    ar_cd,
                    shp_area,
                    prv_cd,
                    aoj_sz,
                    reg,
                    name,
                    gons,
                };
                if let Some(aojs) = ar_aojs.get_mut(&aoj.ar) {
                    aojs.push(aoj);
                } else {
                    ar_aojs.insert(aoj.ar.to_string(), vec![aoj]);
                }
            } // end loop
            println!("cn: {cn}");
            println!("office === {a1:?}");
            println!("ar_cd === {a2:?}");
            println!("pea === {a3:?}");
            for (ar, aojs) in &mut ar_aojs {
                aojs.sort_by(|a, b| a.ar_cd.cmp(&b.ar_cd));
                let fout = format!("/mnt/e/CHMBACK/pea-data/data1/p12_{ar}_aoj.bin");
                println!("{ar} write to {fout}");
                if let Ok(ser) = bincode::serialize(&aojs) {
                    std::fs::write(fout, ser)?;
                }
            }
        }
    }
    Ok(())
}
*/


