use crate::utl2::attr;
use crate::utl2::attr_map;
use crate::utl2::XTG_BENDPOINT;
use crate::utl2::XTG_BOUNDS;
use crate::utl2::XTG_CHILD;
use crate::utl2::XTG_CONTENT;
use crate::utl2::XTG_DOCUMENTATION;
use crate::utl2::XTG_ELEMENT;
use crate::utl2::XTG_FEATURE;
use crate::utl2::XTG_FOLDER;
use crate::utl2::XTG_HINT_CONTENT;
use crate::utl2::XTG_MODEL;
use crate::utl2::XTG_PROFILE;
use crate::utl2::XTG_PROPERTY;
use crate::utl2::XTG_SOURCE_CONNECNTION;
use crate::utl3::ass_var_aoj;
use crate::utl3::ass_var_aoj_tr;
use std::io::Write;
use zip::write::SimpleFileOptions;
use crate::utl2::ScriptParam;
use bincode::{Decode, Encode};
use std::sync::{LazyLock, Mutex};
use crate::dcl::BranchGIS;
use crate::dcl::PeaSub;
use crate::dcl::Pan;
use crate::dcl::PeaAssVar;
use crate::dcl::VarType;
use crate::img::fda01::get_img;
use image::ImageReader;
use std::io::Read;
use strum::IntoEnumIterator;
use quick_xml::events::BytesStart;
use crate::asm::ASM;
use quick_xml::events::Event;
use quick_xml::reader::Reader as XmlReader;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::io;
use std::rc::Rc;
use docx_rs::*;

#[derive(Debug, Clone, Default)]
pub enum NumValEnum {
    Real(f32),
    Int(i32),
    Text(String),
    Json(Value),
    #[default]
    None,
}

#[derive(Debug, Clone, Default)]
pub struct AssumObject {
    pub name: String,
    pub dlgid: String,
    pub elmid: String,
    pub tagid: String,
}
#[derive(Debug, Clone, Default)]
pub struct AssumValue {
    pub sid: String,
    pub val: NumValEnum,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct Archi {
    pub asm: HashMap<String, NumValEnum>,
    pub view_name: String,
    pub vwid: String,
    pub asmrv: Vec<f32>,
    pub asmiv: Vec<i32>,
    pub asmtv: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ArchElem {
    pub elem: String,
    pub id: String,
    pub text: String,
    pub attr: HashMap<String, String>,
    pub names: String,
    pub child: Vec<Rc<RefCell<ArchElem>>>,
    pub elmhm: HashMap<String, Rc<RefCell<ArchElem>>>,
}


impl ArchElem {
    pub fn new(e: &BytesStart, namev: &[String]) -> Self {
        let (i, j) = (1, 10);
        let names = namev
            .iter()
            .enumerate()
            .filter(|(ii, _)| *ii >= i && *ii <= j)
            .map(|(_, d)| d.to_owned())
            .collect::<Vec<String>>()
            .join("/");
        let attr = attr_map(e);
        let elem = String::from_utf8(e.local_name().as_ref().to_vec()).unwrap();
        let id = "".to_string();
        let id = attr.get("id").unwrap_or(&id).to_string();
        ArchElem {
            elem,
            attr: attr_map(e),
            id,
            names,
            ..Default::default()
        }
    }
}

/*
pub fn vec_ij(v: &[String], i: usize, j: usize) -> String {
    v.iter()
        .enumerate()
        .filter(|(ii, _)| *ii >= i && *ii <= j)
        .map(|(_, d)| d.to_owned())
        .collect::<Vec<String>>()
        .join("/")
}
*/

impl Archi {
    pub fn ve(&self, asm: ASM) -> Result<NumValEnum, Box<dyn Error>> {
        let asss = format!("{asm:?}");
        if let Some(ass) = self.asm.get(&asss) {
            return Ok(ass.clone());
        }
        Err("No Assumption - {ss:?}".into())
    }
    pub fn vv(&self, asm: ASM) -> Result<f32, Box<dyn Error>> {
        let asss = format!("{asm:?}");
        if let Some(ass) = self.asm.get(&asss) {
            if let NumValEnum::Real(a) = ass {
                return Ok(*a);
            }
        }
        Err("No Assumption - {ss:?}".into())
    }
    pub fn v(&self, asm: ASM) -> f32 {
        self.asmrv[asm as usize]
    }
    pub fn u(&self, asm: ASM) -> usize {
        self.asmiv[asm as usize] as usize
    }
    pub fn t(&self, asm: ASM) -> String {
        self.asmtv[asm as usize].to_string()
    }
}


pub fn archi6(vnm: &str) -> Result<(), Box<dyn Error>> {
    let arc = make_archi(vnm)?;
    for (i, asm) in ASM::iter().enumerate() {
        let n = format!("{asm:?}");
        let v = arc.v(asm.clone());
        let t = arc.t(asm);
        println!("{i}: {n} => {v} - {t}");
    }
    Ok(())
}

pub const C_ARCHFILE: &str = "/mnt/c/Users/num/wk33/peasg/archi3/PEA-SG-admin-2025-12-25.archimate";
pub const E_ARCHFILE: &str = "/mnt/e/CHMBACK/pea-data/archi/PEA-SG-admin-2025-12-25.archimate";

fn get_view_name(vid: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let vws = vid.split(":");
    let mut wds = vec![];
    for w in vws {
        wds.push(w);
    }
    if wds.len() < 2 {
        println!("Put view name c:view name");
        return Err("Error 1".into());
    }
    //println!("wds:{wds:?}");
    let d = match wds[0] {
        "C" => C_ARCHFILE,
        "E" => E_ARCHFILE,
        _ => return Err("Error 2".into()),
    };
    let v = wds[1];
    Ok((d, v))
    //Ok(format!("{d}/{}", wds[1]))
}

pub fn ass_str_to_enum(ass: &HashMap<String, NumValEnum>) -> Vec<f32> {
    let mut cn = 0;
    let mut s2n = HashMap::<String, usize>::new();
    for a in ASM::iter() {
        cn += 1;
        let an = format!("{a:?}");
        let ai = a as usize;
        s2n.insert(an.to_string(), ai);
        //println!("{i}. {ai} {an}");
    }
    let mut vnum = vec![0f32; cn];
    for (k, v) in ass.iter() {
        let kk = k.to_string();
        let Some(ai) = s2n.get(&kk) else {
            continue;
        };
        let NumValEnum::Real(v) = v.clone() else {
            continue;
        };
        vnum[*ai] = v;
        //println!("{ai} {v:?}");
    }
    vnum
}

impl Archi {
    pub fn init1(&mut self) {
        let mut cn = 0;
        let mut s2n = HashMap::<String, usize>::new();
        for a in ASM::iter() {
            cn += 1;
            let an = format!("{a:?}");
            let ai = a as usize;
            s2n.insert(an.to_string(), ai);
            //println!("{i}. {ai} {an}");
        }
        self.asmrv = vec![0f32; cn];
        self.asmiv = vec![0i32; cn];
        self.asmtv = vec!["".to_string(); cn];
        for (k, v) in self.asm.iter() {
            let kk = k.to_string();
            let Some(ai) = s2n.get(&kk) else {
                continue;
            };
            match v {
                NumValEnum::Real(v) => {
                    self.asmrv[*ai] = *v;
                }
                NumValEnum::Int(v) => {
                    self.asmiv[*ai] = *v;
                }
                NumValEnum::Text(v) => {
                    self.asmtv[*ai] = v.to_string();
                }
                _ => {}
            }
        }
    }
}

pub fn make_archi(vid: &str) -> Result<Archi, Box<dyn Error>> {
    let (af, vn) = get_view_name(vid)?;
    //println!("af:{af} vn:{vn}");
    let outdir = "archi-ex";
    archi_extract(af, outdir)?;
    let arc_an = archi_ana1(outdir, vn)?;
    //println!("vn:{}, {:?}", arc_an.view_name, arc_an.vnids);
    if arc_an.vnids.is_empty() {
        return Err("View not found".into());
    }
    if arc_an.vnids.len() > 1 {
        return Err(format!("Multiple view {:?}", arc_an.vnids).into());
    }
    let vid = arc_an.vnids[0].as_str();
    let asm = make_assum_val(vid, &arc_an);
    let mut arc = Archi {
        asm,
        view_name: vn.to_string(),
        ..Default::default()
    };
    arc.init1();
    //archi_init1(&mut arc);
    //let asmrv = ass_str_to_enum(&asm);
    Ok(arc)
}

pub fn make_archi2(vid: &str) -> Result<(Archi, ArchiAnalyze), Box<dyn Error>> {
    let (af, vn) = get_view_name(vid)?;
    //println!("af:{af} vn:{vn}");
    let outdir = "archi-ex";
    archi_extract(af, outdir)?;
    let arc_an = archi_ana1(outdir, vn)?;
    //println!("vn:{}, {:?}", arc_an.view_name, arc_an.vnids);
    if arc_an.vnids.is_empty() {
        return Err("View not found".into());
    }
    if arc_an.vnids.len() > 1 {
        return Err(format!("Multiple view {:?}", arc_an.vnids).into());
    }
    let vid = arc_an.vnids[0].as_str();
    let asm = make_assum_val(vid, &arc_an);
    let mut arc = Archi {
        asm,
        view_name: vn.to_string(),
        vwid: vid.to_string(),
        ..Default::default()
    };
    arc.init1();
    //archi_init1(&mut arc);
    //let asmrv = ass_str_to_enum(&asm);
    Ok((arc, arc_an))
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

#[derive(Debug, Clone)]
pub struct DiagInfo {
    pub bnd: Rect,
    pub did: String,
    pub eid: String,
}

pub fn get_elem_rect(el: &std::cell::Ref<ArchElem>) -> Result<Rect, Box<dyn Error>> {
    for ch in &el.child {
        let ch = ch.borrow();
        if ch.elem == "bounds" {
            let x0 = ch.attr.get("x").unwrap().parse::<i32>()?;
            let y0 = ch.attr.get("y").unwrap().parse::<i32>()?;
            let x1 = x0 + ch.attr.get("width").unwrap().parse::<i32>()?;
            let y1 = y0 + ch.attr.get("height").unwrap().parse::<i32>()?;
            return Ok(Rect { x0, y0, x1, y1 });
        }
    }
    Err("Error".into())
}

pub fn all_diags_in_view(
    e: &std::cell::Ref<ArchElem>,
    _arc: &ArchiAnalyze,
) -> (Vec<DiagInfo>, Vec<DiagInfo>) {
    let mut diavw = Vec::<DiagInfo>::new();
    let mut diaem = Vec::<DiagInfo>::new();
    //println!("count elem: {}", e.child.len());
    let mut memb = vec![];
    for ch in e.child.iter() {
        memb.push(ch.clone());
    }
    while let Some(ch) = memb.pop() {
        let ch = ch.borrow();
        let Some(tp) = ch.attr.get("xsi:type") else {
            continue;
        };
        let Some(did) = ch.attr.get("id") else {
            continue;
        };
        let Ok(bnd) = get_elem_rect(&ch) else {
            continue;
        };
        let nstr = String::new();
        match tp.as_str() {
            "archimate:DiagramModelReference" => {
                let eid = ch.attr.get("model").unwrap_or(&nstr);
                let dia = DiagInfo {
                    bnd: bnd.clone(),
                    eid: eid.to_string(),
                    did: did.to_string(),
                };
                diavw.push(dia.clone());
            }
            "archimate:DiagramObject" => {
                let eid = ch.attr.get("archimateElement").unwrap_or(&nstr);
                diaem.push(DiagInfo {
                    bnd: bnd.clone(),
                    eid: eid.to_string(),
                    did: did.to_string(),
                });
                for ch in ch.child.iter() {
                    memb.push(ch.clone());
                }
                //println!(" diag:{}", ch.child.len());
            }
            _ => {}
        }
    }
    //println!("vw:{} el:{}", diavw.len(), diaem.len());
    (diavw, diaem)
}

pub fn collect_diags_in_view(
    e: &std::cell::Ref<ArchElem>,
    _es: &HashMap<String, Rc<RefCell<ArchElem>>>,
) -> (Vec<DiagInfo>, Vec<DiagInfo>, Vec<DiagInfo>) {
    let mut diavw = Vec::<DiagInfo>::new();
    let mut diaem = Vec::<DiagInfo>::new();
    let mut diaal = Vec::<DiagInfo>::new();
    println!("count elem: {}", e.child.len());
    for (_i, ch) in e.child.iter().enumerate() {
        //let rc = ch.clone();
        let ch = ch.borrow();
        let Some(tp) = ch.attr.get("xsi:type") else {
            continue;
        };
        let Some(did) = ch.attr.get("id") else {
            continue;
        };
        let Ok(bnd) = get_elem_rect(&ch) else {
            return (diavw, diaem, diaal);
        };
        let nstr = String::new();
        match tp.as_str() {
            "archimate:DiagramModelReference" => {
                let eid = ch.attr.get("model").unwrap_or(&nstr);
                let dia = DiagInfo {
                    bnd: bnd.clone(),
                    eid: eid.to_string(),
                    did: did.to_string(),
                };
                diavw.push(dia.clone());
                diaal.push(DiagInfo {
                    bnd: bnd.clone(),
                    eid: eid.to_string(),
                    did: did.to_string(),
                });
            }
            "archimate:DiagramObject" => {
                let eid = ch.attr.get("archimateElement").unwrap_or(&nstr);
                diaem.push(DiagInfo {
                    bnd: bnd.clone(),
                    eid: eid.to_string(),
                    did: did.to_string(),
                });
                diaal.push(DiagInfo {
                    bnd: bnd.clone(),
                    eid: eid.to_string(),
                    did: did.to_string(),
                });
            }
            _ => {}
        }
    }
    (diavw, diaem, diaal)
}

fn get_child<'a>(ee: &'a ArchElem, tg: &'a str) -> Result<Rc<RefCell<ArchElem>>, Box<dyn Error>> {
    for ch in ee.child.iter() {
        let cln = ch.clone();
        let che = ch.borrow();
        if che.elem == tg {
            return Ok(cln);
        }
    }
    Err("No tag".into())
}

pub fn repo1(vwnm: &str, _fnm: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("vnm: {vwnm}");
    let (ac, aa) = crate::utl4::make_archi2(vwnm)?;
    println!("vw:{} fd:{} el:{}", aa.view_name, aa.ar_fold.child.len(), aa.ar_elem.child.len());
    let dnm = ac.t(ASM::OUTDIR);
    let fnm = ac.t(ASM::OUTFILE);
    crate::dcl::set_dirnm(&dnm);
    println!("dnm: {dnm} fnm: {fnm}");
    let mut docv = create_docx_def0();
    docv = ar_gen_doc(docv, ac.vwid.as_str(), &aa.elmhm)?;
    let dout = format!("{dnm}/out");
    let fnm1 = format!("{dout}/{fnm}.docx");
    let fnm2 = format!("{dout}/{fnm}-1.docx");
    fs::create_dir_all(dout).unwrap();
    write_docx(docv, fnm2.as_str(), fnm1.as_str(), "temp")?;
    Ok(())
}

pub fn make_assum_val(
    vid: &str,
    arc: &ArchiAnalyze, //elem: &HashMap<String, Rc<RefCell<ArchElem>>>,
) -> HashMap<String, NumValEnum> {
    //println!("============= VIEWID: {vid}");
    let mut ass_m = HashMap::<String, NumValEnum>::new();
    if let Some(e) = arc.elmhm.get(vid) {
        let e = e.borrow();
        let (diavw, diaem) = all_diags_in_view(&e, arc);
        //println!("d:{} v:{}", diaem.len(), diavw.len());
        let mut ass_obj = Vec::<AssumObject>::new();
        let mut ass_val = Vec::<AssumValue>::new();

        for dg in diaem.iter() {
            //println!("{g}. eid:{} did:{}", dg.eid, dg.did);
            //let Some(e) = &arc.elmhm.get(&dg.eid) else {
            let Some(ee) = &arc.elmhm.get(&dg.eid) else {
                continue;
            };
            let Some(de) = &arc.ar_memb.elmhm.get(&dg.did) else {
                continue;
            };
            let ee = ee.borrow();
            let de = de.borrow();
            let Some(xtp) = ee.attr.get("xsi:type") else {
                continue;
            };
            let Some(prfid) = ee.attr.get("profiles") else {
                continue;
            };
            let Some(prfnm) = arc.prfhm.get(prfid) else {
                continue;
            };
            let Some(name) = ee.attr.get("name") else {
                continue;
            };
            let mut doc = "".to_string();
            for e in ee.child.iter() {
                let e = e.borrow();
                if e.elem.as_bytes() == XTG_DOCUMENTATION {
                    doc = e.text.to_string();
                }
            }
            //println!("    xtp:{xtp} prfnm:{prfnm} name;{name}");
            if *xtp == "archimate:BusinessObject" && prfnm == "Assum" {
                let Some(tids) = de.attr.get("targetConnections") else {
                    continue;
                };
                //println!(" BusObject: {tids}");
                let asob = AssumObject {
                    name: name.to_string(),
                    dlgid: dg.did.to_string(),
                    elmid: dg.eid.to_string(),
                    tagid: tids.to_string(),
                };
                ass_obj.push(asob);
            }
            if *xtp == "archimate:Artifact" {
                let Ok(ssel) = get_child(&de, "sourceConnection") else {
                    continue;
                };
                let ssel = ssel.borrow();
                let Some(ssid) = ssel.attr.get("id") else {
                    continue;
                };
                match prfnm.as_str() {
                    "AssumNum" => {
                        let name = name.replace(",", "");
                        let name = name.replace("_", "");
                        let name = name.replace("f32", "");
                        let valu = if let Ok(vf32) = name.parse::<f32>() {
                            NumValEnum::Real(vf32)
                        } else {
                            NumValEnum::None
                        };
                        //println!(" AssumValue: {ssid}");
                        let asva = AssumValue {
                            sid: ssid.to_string(),
                            name: name.clone(),
                            val: valu,
                        };
                        ass_val.push(asva);
                    }
                    "AssumText" => {
                        let valu = NumValEnum::Text(name.clone());
                        //println!("============= SSID: {} [{valu:?}]", ssid);
                        let asva = AssumValue {
                            sid: ssid.to_string(),
                            name: name.clone(),
                            val: valu,
                        };
                        ass_val.push(asva);
                    }
                    "AssumJson" => {
                        let mut name = name.replace("&quot;", "\"");
                        if name == "value" {
                            name = doc;
                        }
                        match serde_json::from_str::<Value>(name.as_str()) {
                            Ok(v) => {
                                let val = NumValEnum::Json(v.clone());
                                //println!(">>>>>>>>>>>>>>>>>>>>>>> JSON {name:?} {v:?}");
                                /*
                                if let Value::Array(v) = v {
                                    println!("   ARRAY: {}", v.len());
                                }
                                */
                                let asva = AssumValue {
                                    sid: ssid.to_string(),
                                    name: name.clone(),
                                    val,
                                };
                                ass_val.push(asva);
                            }
                            Err(e) => {
                                println!(">>>>>>>>>>>> JSON {name:?} {e:?}");
                            }
                        }
                    }
                    ass => {
                        println!("====== NO ASSUM {ass:?}");
                    }
                }
            }
        }
        //println!("nms:{} tgs:{}", ass_obj.len(), ass_val.len());
        let mut obnms = HashSet::<String>::new();
        let mut obtgs = HashMap::<String, AssumObject>::new();
        //println!("============ OBJECT ===========");
        for ob in ass_obj.iter() {
            //println!("{i:2}. OBJECT: {} {}", ob.name, ob.tagid);
            if let Some(ob) = obnms.get(&ob.name) {
                println!("================ ASSU OBJ DUPLICATE : {ob:?}");
            }
            obnms.insert(ob.name.clone());
            for tg in ob.tagid.split(" ") {
                let tg = tg.to_string();
                obtgs.insert(tg, ob.clone());
            }
            //obtgs.insert(ob.tagid.clone(), ob.clone());
        }
        //println!("============ VALUE ===========");
        for (_i, va) in ass_val.iter().enumerate() {
            if let Some(ob) = obtgs.get(&va.sid) {
                //println!("ASSUM {} - {:?}", ob.name, va.val);
                ass_m.insert(ob.name.clone(), va.val.clone());
            }
        }
        //println!("vws:{}", diavw.len());
        for vw in diavw.iter() {
            let mut asmhm = make_assum_val(&vw.eid, arc);
            //println!("===== vw: {} = {}", vw.eid, asmhm.len());
            for (k, v) in asmhm.iter_mut() {
                if let Some(ov) = ass_m.get(k) {
                    println!("duplicate {k} old:{ov:?} new:{v:?}");
                } else {
                    ass_m.insert(k.to_string(), v.clone());
                }
            }
        }
    }
    ass_m
}

fn archi_extract(indir: &str, outdir: &str) -> Result<(), Box<dyn Error>> {
    //println!("in file: {indir}");
    let parch = std::path::Path::new(indir);
    let farch = fs::File::open(parch).unwrap();
    let mut arch = zip::ZipArchive::new(farch).unwrap();

    //println!("outdir: {outdir}");
    //let img_dir = format!("{}/images/", outdir);
    fs::create_dir_all(outdir).unwrap();
    for i in 0..arch.len() {
        let mut file = arch.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        if file.is_dir() {
            let dir = format!("{}/{}", outdir, outpath.display());
            fs::create_dir_all(&dir).unwrap();
            //println!("dir: {dir}");
        } else {
            let fnm = format!("{}/{}", outdir, outpath.display());
            if fnm.ends_with(".xml") {
                //println!("save: {fnm}");
                let mut outfile = fs::File::create(&fnm).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
                break;
            }
        }
    }
    Ok(())
}

pub struct ArchiAnalyze {
    elmhm: HashMap<String, Rc<RefCell<ArchElem>>>,
    view_name: String,
    vnids: Vec<String>,
    prfhm: HashMap<String, String>,
    ar_fold: ArchElem,
    ar_elem: ArchElem,
    ar_memb: ArchElem,
}

fn archi_ana1(outdir: &str, vn: &str) -> Result<ArchiAnalyze, Box<dyn Error>> {
    //) -> Result<HashMap<String, Rc<RefCell<ArchElem>>>, Box<dyn Error>> {
    let fmod = format!("{outdir}/model.xml");
    if let Ok(mut xrd) = XmlReader::from_file(&fmod) {
        //println!("XML OPEN {fmod}");
        let mut xbuf = Vec::new();
        let mut pathv = Vec::<String>::new();
        let mut namev = Vec::<String>::new();
        let mut ar_elem = ArchElem {
            elem: String::from("element"),
            ..Default::default()
        };
        let mut ar_fold = ArchElem {
            elem: String::from("folder"),
            ..Default::default()
        };
        let mut ar_memb = ArchElem {
            elem: String::from("child"),
            ..Default::default()
        };
        let mut ar_stack = Vec::<Rc<RefCell<ArchElem>>>::new();
        //let mut cn = 0;
        let mut vnids = vec![];
        loop {
            //cn += 1;
            match xrd.read_event_into(&mut xbuf) {
                Ok(Event::Eof) => break,
                Ok(Event::Empty(e)) => {
                    match e.local_name().as_ref() {
                        XTG_MODEL => {
                            //ar_elem.attr = attr;
                            ar_elem.attr = attr_map(&e);
                        }
                        XTG_FOLDER => {}
                        XTG_ELEMENT | XTG_PROFILE => {
                            let el = ArchElem::new(&e, &namev);
                            //el.attr = attr;
                            let ee = Rc::new(RefCell::new(el));
                            ar_elem.child.push(ee);
                            if ar_fold.child.is_empty() {
                                println!("EMPTY FOLDER");
                            }
                        }
                        XTG_CHILD => {
                            println!("  EMPTY CHILD");
                        }
                        XTG_PROPERTY
                        | XTG_BOUNDS
                        | XTG_SOURCE_CONNECNTION
                        | XTG_BENDPOINT
                        | XTG_FEATURE
                        | XTG_CONTENT
                        | XTG_HINT_CONTENT
                        | XTG_DOCUMENTATION => {
                            let el = ArchElem::new(&e, &namev);
                            //el.attr = attr;
                            if !ar_stack.is_empty() {
                                let ii = ar_stack.len() - 1;
                                let ar = &mut ar_stack[ii];
                                let mut ar = ar.borrow_mut();
                                let ee1 = Rc::new(RefCell::new(el));
                                ar.child.push(ee1);
                            } else {
                                println!("E2: {}", el.elem);
                            }
                        }
                        e => {
                            println!(
                                " error empty element - {}",
                                String::from_utf8(e.to_vec()).unwrap()
                            );
                        }
                    }
                }
                Ok(Event::Start(e)) => {
                    let e_name = String::from_utf8(e.local_name().as_ref().to_vec()).unwrap();
                    let a_name = attr(&e, b"name");
                    pathv.push(e_name.clone());
                    if let Some(a) = a_name {
                        namev.push(a.clone());
                    } else {
                        //println!("NO NAME {namev:?}");
                        namev.push("?".to_string());
                    }
                    //let attr = attr_map(&e);
                    match e.local_name().as_ref() {
                        XTG_MODEL => {
                            //ar_elem.attr = attr;
                            ar_elem.attr = attr_map(&e);
                        }
                        XTG_FOLDER => {
                            let el = ArchElem::new(&e, &namev);
                            //el.attr = attr;
                            if let Some(_id) = el.attr.get("id") {
                            } else {
                                println!(
                                    " NO ID : {}",
                                    String::from_utf8(e.local_name().as_ref().to_vec()).unwrap()
                                );
                            }
                            let ee1 = Rc::new(RefCell::new(el));
                            let ee2 = ee1.clone();
                            ar_fold.child.push(ee2);
                            let ee3 = ee1.clone();
                            if !ar_stack.is_empty() {
                                let ii = ar_stack.len() - 1;
                                let ar = &mut ar_stack[ii];
                                let mut ar = ar.borrow_mut();
                                ar.child.push(ee3);
                            }
                            ar_stack.push(ee1);
                        }
                        XTG_ELEMENT => {
                            let el = ArchElem::new(&e, &namev);
                            if let (Some(xtp), Some(id), Some(nm)) = (
                                el.attr.get("xsi:type"),
                                el.attr.get("id"),
                                el.attr.get("name"),
                            ) {
                                if xtp == "archimate:ArchimateDiagramModel" && nm == vn {
                                    vnids.push(id.clone());
                                    //println!("2: xtp:{xtp} id:{id} nm:{nm}");
                                }
                            }
                            let ee1 = Rc::new(RefCell::new(el));
                            let ee2 = ee1.clone();
                            let ee3 = ee1.clone();
                            ar_elem.child.push(ee1);
                            if ar_stack.is_empty() {
                                println!("ERROR4: ");
                            } else {
                                let ii = ar_stack.len() - 1;
                                let ar = &mut ar_stack[ii];
                                let mut ar = ar.borrow_mut();
                                ar.child.push(ee3);
                            }
                            ar_stack.push(ee2);
                        }
                        XTG_CHILD => {
                            let el = ArchElem::new(&e, &namev);
                            if let Some(_id) = el.attr.get("id") {
                            } else {
                                println!(
                                    " NO ID : {}",
                                    String::from_utf8(e.local_name().as_ref().to_vec()).unwrap()
                                );
                            }
                            //let id = "".to_string();
                            //let id = el.attr.get("id").unwrap_or(&id).to_string();
                            let elm = el.elem.to_string();
                            let ee1 = Rc::new(RefCell::new(el));
                            let ee2 = ee1.clone();
                            let ee3 = ee1.clone();
                            ar_memb.child.push(ee3);
                            if ar_stack.is_empty() {
                                println!("E3: {} {:?}", elm, namev);
                            } else {
                                let ii = ar_stack.len() - 1;
                                let ar = &mut ar_stack[ii];
                                let mut ar = ar.borrow_mut();
                                ar.child.push(ee1);
                            }
                            ar_stack.push(ee2);
                        }
                        XTG_CONTENT
                        | XTG_HINT_CONTENT
                        | XTG_SOURCE_CONNECNTION
                        | XTG_DOCUMENTATION => {
                            let el = ArchElem::new(&e, &namev);
                            //el.attr = attr;
                            let elm = el.elem.to_string();
                            let ee1 = Rc::new(RefCell::new(el));
                            let ee2 = ee1.clone();
                            if ar_stack.is_empty() {
                                println!("E3: {} {:?}", elm, namev);
                            } else {
                                let ii = ar_stack.len() - 1;
                                let ar = &mut ar_stack[ii];
                                let mut ar = ar.borrow_mut();
                                ar.child.push(ee1);
                            }
                            ar_stack.push(ee2);
                        }
                        e => {
                            println!(" error - {}", String::from_utf8(e.to_vec()).unwrap());
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    pathv.pop();
                    namev.pop();
                    match e.local_name().as_ref() {
                        XTG_MODEL => {}
                        XTG_FOLDER => {
                            ar_stack.pop();
                        }
                        XTG_ELEMENT => {
                            ar_stack.pop();
                        }
                        XTG_CHILD
                        | XTG_CONTENT
                        | XTG_HINT_CONTENT
                        | XTG_SOURCE_CONNECNTION
                        | XTG_DOCUMENTATION => {
                            ar_stack.pop();
                        }
                        e => {
                            println!("ERROR 3 : {}", String::from_utf8(e.to_vec()).unwrap());
                        }
                    }
                }
                Ok(Event::Text(tx)) => {
                    let tx = String::from_utf8(tx.to_vec()).unwrap_or_default();
                    let tx = tx.trim();
                    if !tx.is_empty() {
                        {
                            let ii = ar_stack.len() - 1;
                            let ar = &mut ar_stack[ii];
                            let mut ar = ar.borrow_mut();
                            //ar.text = tx.to_string();
                            ar.text = format!("{}{}", ar.text, tx);
                            //println!("   DOC TEXT e:'{}' - t:''", ar.elem);
                        }
                    }
                }
                Ok(Event::GeneralRef(ge)) => {
                    let ge = String::from_utf8(ge.to_vec()).unwrap_or_default();
                    let tx = match ge.as_str() {
                        "#xD" => "\n".to_string(),
                        "lt" => "<".to_string(),
                        "gt" => "<".to_string(),
                        "amp" => "&".to_string(),
                        "quot" => "\"".to_string(),
                        "apos" => "\'".to_string(),
                        e => {
                            println!("{e:?}");
                            e.to_string()
                        }
                    };
                    if !tx.is_empty() {
                        {
                            let ii = ar_stack.len() - 1;
                            let ar = &mut ar_stack[ii];
                            let mut ar = ar.borrow_mut();
                            //ar.text = tx.to_string();
                            ar.text = format!("{}{}", ar.text, tx);
                            //println!("   DOC TEXT e:'{}' - t:''", ar.elem);
                        }
                    }
                }
                Ok(Event::Decl(_de)) => {
                    //println!("========= Decl : {:?}", _de);
                }
                e => {
                    println!("========= ERROR 2 : {:?}", e);
                }
            }
            xbuf.clear();
        } // XML loop
        let mut ar_elmh = HashMap::<String, Rc<RefCell<ArchElem>>>::new();
        let mut prfhm = HashMap::<String, String>::new();
        for e in ar_elem.child.iter() {
            let ee = e.borrow();
            let Some(id) = ee.attr.get("id") else {
                continue;
            };
            if ee.elem == "profile" {
                let Some(nm) = ee.attr.get("name") else {
                    continue;
                };
                prfhm.insert(id.to_string(), nm.to_string());
            }
            let mut idhs = HashSet::<String>::new();
            for ch in ee.child.iter() {
                let ech = ch.borrow();
                if let Some(id) = ech.attr.get("id") {
                    if let Some(_id) = idhs.get(id) {
                        println!("==== ID dup {id}");
                    }
                    idhs.insert(id.clone());
                }
            }
            ar_elmh.insert(id.to_string(), e.clone());
        }
        //println!("cnt:{cn}");
        for elem in [&mut ar_elem, &mut ar_fold, &mut ar_memb] {
            //let elem = &mut ar_elem;
            for e in elem.child.iter() {
                let c = e.clone();
                let e = e.borrow();
                let Some(id) = e.attr.get("id") else {
                    continue;
                };
                elem.elmhm.entry(id.to_string()).or_insert(c);
            }
            //println!(" ???? {}", elem.elmhm.len());
        }
        let ac = ArchiAnalyze {
            elmhm: ar_elmh,
            view_name: vn.to_string(),
            vnids,
            prfhm,
            ar_elem,
            ar_fold,
            ar_memb,
        };
        return Ok(ac);
    } // open XML file
    Err("Failure".into())
}

pub fn get_elem_prop(el: &std::cell::Ref<ArchElem>, p_key: &str) -> Vec<String> {
    let mut props = Vec::<String>::new();
    for ch in &el.child {
        let ch = ch.borrow();
        if ch.elem == "property"
            && let Some(ky) = ch.attr.get("key")
            && ky == p_key
            && let Some(va) = ch.attr.get("value")
        {
            props.push(va.to_string());
        }
    }
    props
}

pub fn get_elem_doc(el: &std::cell::Ref<ArchElem>) -> String {
    for ch in &el.child {
        let ch = ch.borrow();
        if ch.elem == "documentation" {
            let tx = ch.text.to_string();
            return tx;
        }
    }
    String::new()
}

pub const STD_FONT: &str = "TH Sarabun New";

#[derive(Debug, Clone)]
pub enum DiagramType {
    None,
    ElemRef,
    ViewRef,
    NoteRef,
}

#[derive(Debug, Clone)]
pub struct Bound {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

#[derive(Debug, Clone)]
pub struct Diagram {
    pub eid: String,
    pub dgtp: DiagramType,
    pub bnd: Bound,
    pub rc: Rc<RefCell<ArchElem>>,
    pub xtp: String,
    pub prf: String,
    pub vtp: String,
}

pub fn check(bnd: &Bound, vw: &ArViewDocInfo) {
    println!("{bnd:?} {vw:?}");
    println!("{}", bnd.y1);
    println!("{}", vw.vnm);
}

pub fn txt_lines(doc: String) -> Vec<String> {
    let parts = doc.split("\n");
    let mut line = String::new();
    let mut lines = Vec::<String>::new();
    for ln in parts {
        let ll = ln.to_string();
        if ll.is_empty() {
            if !line.is_empty() {
                lines.push(line);
            }
            line = String::new();
        }
        line = format!("{line}{ll}");
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

pub fn ar_cmd_split(line: &str) -> Vec<String> {
    let mut pms = Vec::<String>::new();
    let pre = regex::Regex::new(r"\|([^\|]+)").unwrap();
    for caps in pre.captures_iter(line) {
        if let Some(cap) = caps.get(1) {
            let a = cap.as_str();
            pms.push(a.to_string());
        }
    }
    if pms.is_empty() {
        pms.push(line.to_string());
    }
    pms
}
//======================================================
pub fn footnote(
    mut docv: Vec<DocxCompo>,
    _vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let c1 = Paragraph::new().add_run(
        Run::new()
            .add_footnote_reference(
                Footnote::new()
                    .add_content(Paragraph::new().add_run(Run::new().add_text("EA: id name"))),
            )
            .size(20)
            .fonts(RunFonts::new().cs(STD_FONT)),
    );
    //.indent(Some(840), None, None, None);
    docv.push(DocxCompo::Paragraph(c1));
    Ok(docv)
}

//======================================================
pub fn doc0(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    println!("doc0 {lst}");
    let c1 = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    //docv = add_docv(docv, c1, vw, ft)?;
    docv = add_docxv(docv, DocxCompo::Paragraph(c1), vw)?;
    Ok(docv)
}

//======================================================
pub fn dn(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    n: i32,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    let idn = n * 700;
    let n = n as usize;
    println!("doc1 {lst}");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .outline_lvl(n)
        .indent(Some(idn), None, None, None);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn hn(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    n: i32,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    if n == 2 {
        docv = sp(docv, vw)?;
    }
    let lst = pws.last().unwrap();
    let cnt = modi_head(lst.as_str(), n as usize);
    let hd = format!("Heading{n}");
    let fnsz = [48, 44, 40, 36, 32];
    let nn = if n > 5 { 5 } else { n };
    let nn = if nn < 1 { 5 } else { nn };
    let nn = fnsz[nn as usize - 1];
    let mut h1 = Paragraph::new().add_run(
        Run::new()
            .add_text(cnt)
            .size(nn)
            .fonts(RunFonts::new().cs(STD_FONT)),
    );
    h1 = h1.style(&hd);
    if n == 1 {
        h1 = h1.page_break_before(true);
    }
    docv = add_docxv(docv, DocxCompo::Paragraph(h1), vw)?;
    println!("hn {n} - {lst}");
    Ok(docv)
}

//======================================================
pub fn c1(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    println!("c1 {lst}");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    //docv = add_docv(docv, pa, vw, ft)?;
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn c2(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    println!("c2 {lst}");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    //docv = add_docv(docv, pa, vw, ft)?;
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn c3(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    println!("c3 {lst}");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn pg(mut docv: Vec<DocxCompo>, vw: &ArViewDocInfo) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    //let lst = pws.last().unwrap();
    //println!("sp");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(" ")
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .page_break_before(true)
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn sp(mut docv: Vec<DocxCompo>, vw: &ArViewDocInfo) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    //let lst = pws.last().unwrap();
    //println!("sp");
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(" ")
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//pub const VW_IMG_PATH: &str = "/mnt/c/Users/choom/Documents/wk33/peasg/archi3/images/";
pub const DOCX_IMG_FAC: i32 = 9525;

//======================================================
pub fn img1(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    println!("img1 {}", vw.vid);
    let dnm = crate::dcl::get_dirnm();
    let fimg = format!("{dnm}/eaviews/{}.png", vw.vid);
    //let fimg = format!("{VW_IMG_PATH}{}.png", vw.vid);
    if std::path::Path::new(&fimg).exists() {
        println!("file {fimg} exists");
        if let Ok(img) = ImageReader::open(&fimg) {
            //println!("image opened");
            if let Ok(img) = img.decode() {
                //println!("image decoded");
                let (mut w, mut h) = (img.width(), img.height());
                //println!("fimg: {fimg} {w},{h}");
                let mut img = std::fs::File::open(fimg).unwrap();
                let mut buf = Vec::new();
                let _ = img.read_to_end(&mut buf).unwrap();
                //let pic = Pic::new(&buf).size(320 * 9525, 240 * 9525);
                let w0 = 640;
                if w > w0 {
                    h = h * w0 / w;
                    w = w0;
                }
                let pic = Pic::new(&buf).size(w * 9525, h * 9525);
                //let ppic = Paragraph::new().add_run(Run::new().add_image(pic.clone()));
                let ppic = Paragraph::new()
                    .add_run(Run::new().add_image(pic.clone()))
                    .align(AlignmentType::Center);
                docv = add_docxv(docv, DocxCompo::Paragraph(ppic), vw)?;
            }
        }
        let lst = modi_fig(lst.as_str());
        let pa = Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(lst)
                    .size(32)
                    .fonts(RunFonts::new().cs(STD_FONT)),
            )
            .align(AlignmentType::Center);
        docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    } else {
        println!("IMG {fimg} not found");
    }
    Ok(docv)
}


//======================================================
pub fn img2(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    println!("IMG2 {pws:?}");
    let sc = script_param(pws);
    let lst = pws.last().unwrap();
    let mut fdid = "KLO03".to_string();
    if sc.fdid.len() == 5 {
        fdid = sc.fdid.to_string();
    }
    let dnm = crate::dcl::get_dirnm();
    let f02 = format!("{dnm}/fdimg-road/{fdid}-rd02.jpeg");
    let buf = get_img(fdid.as_str(), "roadmap", f02.as_str()).unwrap_or_default();
    //println!("buf: {}", buf.len());
    if let Ok(img) = ImageReader::open(&f02) {
        //println!("image opened");
        if let Ok(img) = img.decode() {
            //println!("image decoded");
            let (mut w, mut h) = (img.width(), img.height());
            //println!("  {w},{h}");
            let w0 = 640;
            if w > w0 {
                h = h * w0 / w;
                w = w0;
            }
            let pic = Pic::new(&buf).size(w * 9525, h * 9525);
            let ppic = Paragraph::new()
                .add_run(Run::new().add_image(pic.clone()))
                .align(AlignmentType::Center);
            docv = add_docxv(docv, DocxCompo::Paragraph(ppic), vw)?;
        }
    }
    let lst = modi_fig(lst.as_str());
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .align(AlignmentType::Center);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn img3(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    println!("IMG3 {pws:?}");
    let sc = script_param(pws);
    let lst = pws.last().unwrap();
    let mut fdid = "KLO03".to_string();
    if sc.fdid.len() == 5 {
        fdid = sc.fdid.to_string();
    }
    let dnm = crate::dcl::get_dirnm();
    let f02 = format!("{dnm}/fdimg-sate/{fdid}-rd02.jpeg");
    let buf = get_img(fdid.as_str(), "satellite", f02.as_str()).unwrap_or_default();
    //println!("buf: {}", buf.len());
    if let Ok(img) = ImageReader::open(&f02) {
        //println!("image opened");
        if let Ok(img) = img.decode() {
            //println!("image decoded");
            let (mut w, mut h) = (img.width(), img.height());
            //println!("  {w},{h}");
            let w0 = 640;
            if w > w0 {
                h = h * w0 / w;
                w = w0;
            }
            let pic = Pic::new(&buf).size(w * 9525, h * 9525);
            let ppic = Paragraph::new()
                .add_run(Run::new().add_image(pic.clone()))
                .align(AlignmentType::Center);
            docv = add_docxv(docv, DocxCompo::Paragraph(ppic), vw)?;
        }
    }
    let lst = modi_fig(lst.as_str());
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .align(AlignmentType::Center);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

#[derive(Debug, Clone)]
pub struct ArViewDocInfo<'a> {
    vid: &'a str,
    vnm: String,
    nms: String,
    es: &'a HashMap<String, Rc<RefCell<ArchElem>>>,
    vdia: &'a Vec<Vec<Diagram>>,
    edia: &'a Vec<Vec<Diagram>>,
    adia: &'a Vec<Vec<Diagram>>,
}

pub fn end_ref(docv: Vec<DocxCompo>, vw: &ArViewDocInfo) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let mut ii = 0;
    for (i, doc) in docv.iter().enumerate() {
        if let DocxCompo::Paragraph(_) = doc {
            ii = i;
        }
    }
    let mut vdoc = Vec::<DocxCompo>::new();
    for (i0, doc) in docv.into_iter().enumerate() {
        let doc = if i0 == ii {
            if let DocxCompo::Paragraph(mut par) = doc {
                let v = &vw.nms[6..];
                //let rf = format!("ea: '{v}' {}", vw.vid);
                let rf = format!("ea:{v}");
                par = par.add_run(
                    Run::new()
                        .add_footnote_reference(
                            Footnote::new().add_content(
                                Paragraph::new().add_run(
                                    Run::new()
                                        .add_text(rf)
                                        .size(16)
                                        .fonts(RunFonts::new().cs(STD_FONT)),
                                ),
                            ),
                        )
                        .size(12)
                        .fonts(RunFonts::new().cs(STD_FONT)),
                );
                DocxCompo::Paragraph(par)
            } else {
                doc
            }
        } else {
            doc
        };
        vdoc.push(doc);
    }
    Ok(vdoc)
}

//=======================================================
//
pub fn add_docxv(
    mut docv: Vec<DocxCompo>,
    docx: DocxCompo,
    _vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    docv.push(docx);
    Ok(docv)
}

pub fn data0(_sc: &ScriptParam) -> Vec<Vec<String>> {
    vec![
        vec![
            "1a".to_string(),
            "1b".to_string(),
            "1c".to_string(),
            "1d".to_string(),
        ],
        vec![
            "2a".to_string(),
            "2b".to_string(),
            "2c".to_string(),
            "2d".to_string(),
        ],
    ]
}

pub const DOC_TB_INDENT: i32 = 500;
pub const TABSHADE: &str = "#ecffe6";
pub const CELL_MARGIN: i32 = 100;

//======================================================
pub fn v1(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    println!("view 1");
    let _sc = script_param(pws);
    for dis in vw.vdia.iter() {
        for di in dis.iter() {
            docv = ar_gen_doc(docv, &di.eid, vw.es)?;
        }
    }
    Ok(docv)
}

//======================================================
pub fn v2(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &mut ArViewDocInfo,
    hd: bool,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    println!("view 1");
    let sc = script_param(pws);
    for dis in vw.adia.iter() {
        for di in dis.iter() {
            match di.dgtp {
                DiagramType::ViewRef => {
                    println!("SHOW VIEW >>>>>>>>>>>>>>>>>>>>>>>>");
                    docv = ar_gen_doc(docv, &di.eid, vw.es)?;
                }
                DiagramType::ElemRef => {
                    let mut xtp = if di.xtp.starts_with("archimate:") {
                        di.xtp[10..].to_string()
                    } else {
                        "".to_string()
                    };
                    if !di.prf.is_empty() {
                        xtp = di.prf.to_string();
                    }
                    if sc.xtype.is_empty() || sc.xtype.contains(&xtp) {
                        docv = elm_to_render(docv, di.rc.clone(), vw, hd)?;
                    }
                }
                DiagramType::NoteRef => {}
                DiagramType::None => {}
            }
        }
    }
    Ok(docv)
}

//======================================================
pub fn e2(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &mut ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let _sc = script_param(pws);
    for dis in vw.edia.iter() {
        for di in dis.iter() {
            docv = elm_to_render(docv, di.rc.clone(), vw, true)?;
        }
    }
    Ok(docv)
}

//======================================================
pub fn e1(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let _sc = script_param(pws);
    for dis in vw.edia.iter() {
        for di in dis.iter() {
            if let Some(e) = vw.es.get(&di.eid) {
                let e = e.borrow();
                let vnm = if let Some(n) = e.attr.get("name") {
                    n.to_string()
                } else {
                    "?".to_string()
                };
                let doc = get_elem_doc(&e);
                let doc = format!("{vnm}: {doc}");
                let pws = vec![doc];
                docv = dn(docv, &pws, 2, vw)?;
            }
        }
    }
    Ok(docv)
}

//
//======================================================
pub fn elm_to_render(
    mut docv: Vec<DocxCompo>,
    rc: Rc<RefCell<ArchElem>>,
    vw: &mut ArViewDocInfo,
    hd: bool,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let r = rc.borrow();
    let mut eid = "?".to_string();
    eid = r.attr.get("archimateElement").unwrap_or(&eid).to_string();
    //println!("eid:{eid} chd:{}", r.child.len());
    if let Some(e) = vw.es.get(&eid) {
        let e = e.borrow();
        let enm = "".to_string();
        let enm = e.attr.get("name").unwrap_or(&enm);
        let enm = enm.trim();
        let doc = get_elem_doc(&e);
        let doc = doc.replace("$name", enm);
        //let ses = get_elem_prop(&e, "rdfs:seeAlso");
        if !enm.is_empty() && !doc.is_empty() {
            docv = sp(docv, vw)?;
        }
        if !enm.is_empty() && hd {
            let pa = Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text(enm)
                            .size(32)
                            .bold()
                            .fonts(RunFonts::new().cs(STD_FONT)),
                    )
                    //.indent(Some(idn), None, None, None)
                    ;
            docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
        }
        if !doc.is_empty() {
            let lines = txt_lines(doc);
            docv = ar_line_doc(docv, &lines, vw, false)?;
        }
    }
    for r in &r.child {
        docv = elm_to_render(docv, r.clone(), vw, hd)?;
    }
    Ok(docv)
}
//======================================================
/*
#[derive(Debug, Clone, Default)]
pub struct ScriptParam {
    pub hds: Vec<String>,
    pub mgs: Vec<i32>,
    pub wds: Vec<usize>,
    pub yrs: Vec<usize>,
    pub als: Vec<AlignmentType>,
    pub fld: Vec<String>,
    pub pan: Vec<String>,
    pub lmt: usize,
    pub cmd: Option<String>,
    pub sum: bool,
    pub pvid: String,
    pub sbid: String,
    pub fdid: String,
    pub xtype: Vec<String>,
    pub aojcd: Vec<String>,
    pub max2min: Vec<String>,
    pub min2max: Vec<String>,
}
*/

pub fn script_param(pws: &[String]) -> ScriptParam {
    let prex = regex::Regex::new(r"([0-9a-zA-Z]+)=(.*)$").unwrap();
    let rx_hd = regex::Regex::new(r"([^,]+)").unwrap();
    let rx_wd = regex::Regex::new(r"([0-9][^,]+)").unwrap();
    let rx_al = regex::Regex::new(r"(Left|Center|Right[^,]*)").unwrap();
    let mut sc = ScriptParam::default();
    for w in pws.iter().skip(1).take(pws.len() - 1) {
        let Some(caps) = prex.captures(w) else {
            continue;
        };
        let ky = caps[1].to_string();
        let va = caps[2].to_string();
        //println!("{i}. {w} => {ky} = {va}");
        match ky.as_str() {
            "cmd" => sc.cmd = Some(va),
            "sum" => sc.sum = va == "sum",
            "head" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    //println!("  aa:{aa}");
                    let parts = aa.split(":");
                    let mut p_iter = parts.into_iter();
                    let hd = p_iter.next().unwrap();
                    let cn = if let Some(w) = p_iter.next() {
                        w.parse::<i32>().unwrap()
                    } else {
                        1
                    };
                    //println!("    hd:{hd} cn:{cn}");
                    sc.hds.push(hd.to_string());
                    sc.mgs.push(cn);
                }
            }
            "year" => {
                let nos = va.split(",");
                for n in nos {
                    //println!("  {n}");
                    sc.yrs.push(n.parse::<usize>().unwrap());
                }
            }
            "width" => {
                for caps in rx_wd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    //println!("  aa:{aa}");
                    sc.wds.push(aa.parse::<usize>().unwrap());
                }
            }
            "align" => {
                for caps in rx_al.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    //println!("  aa:{aa}");
                    let al = match aa.as_str() {
                        "Left" => AlignmentType::Left,
                        "Center" => AlignmentType::Center,
                        "Right" => AlignmentType::Right,
                        _ => AlignmentType::Left,
                    };
                    sc.als.push(al);
                }
            }
            "field" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    //println!("  aa:{aa}");
                    let parts = aa.split(":");
                    let mut p_iter = parts.into_iter();
                    let fd = p_iter.next().unwrap();
                    let pn = if let Some(p) = p_iter.next() {
                        p.to_string()
                    } else {
                        "0".to_string()
                    };
                    //println!("    hd:{fd} cn:{pn}");
                    sc.fld.push(fd.to_string());
                    sc.pan.push(pn.to_string());
                }
            }
            "limit" => {
                sc.lmt = va.parse::<usize>().unwrap_or_default();
            }
            "pvid" => {
                sc.pvid = va.to_string();
            }
            "sbid" => {
                sc.sbid = va.to_string();
            }
            "fdid" => {
                sc.fdid = va.to_string();
            }
            "xtype" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    sc.xtype.push(aa);
                }
            }
            "aojcd" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    sc.aojcd.push(aa);
                }
            }
            "max2min" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    sc.max2min.push(aa);
                }
                println!("<<<<<<<<<<<<<<<<<<< MAX2MIN {:?}", sc.max2min);
            }
            "min2max" => {
                for caps in rx_hd.captures_iter(&va) {
                    let aa = caps.get(1).unwrap().as_str().to_string();
                    sc.min2max.push(aa);
                }
                println!("<<<<<<<<<<<<<<<<<<< MIN2MAX {:?}", sc.min2max);
            }
            a => {
                println!("======= UNKNOWN para {a} = {va}");
            }
        }
    }
    sc
}

pub fn fld_2_var(nm: &str) -> VarType {
    for v in VarType::iter() {
        let vs = format!("{v:?}");
        if nm == vs {
            return v;
        }
    }
    VarType::None
}

pub fn val_2_form(v: f32, pn: &str) -> String {
    match pn {
        "1" => v.pan(1),
        "2" => v.pan(2),
        "3" => v.pan(3),
        "P" => {
            let v = v * 100.0;
            format!("{}%", v.pan(1))
        }
        "K" => {
            let v = v / 1_000.0;
            v.pan(1)
        }
        "M" => {
            let v = v / 1_000_000.0;
            v.pan(1)
        }
        _ => v.pan(0),
    }
}

pub fn elm_to_rows(
    mut rows: Vec<Vec<String>>,
    rc: Rc<RefCell<ArchElem>>,
    vw: &mut ArViewDocInfo,
) -> Vec<Vec<String>> {
    let r = rc.borrow();
    let mut eid = "?".to_string();
    eid = r.attr.get("archimateElement").unwrap_or(&eid).to_string();
    if let Some(e) = vw.es.get(&eid) {
        let e = e.borrow();
        let enm = "".to_string();
        let enm = e.attr.get("name").unwrap_or(&enm).to_string();
        let doc = get_elem_doc(&e);
        let no = format!("{}", rows.len());
        if !enm.is_empty() && !doc.is_empty() {
            rows.push(vec![no, enm, doc]);
        }
    }
    for r in &r.child {
        rows = elm_to_rows(rows, r.clone(), vw);
    }
    rows
}

pub fn diag_to_rows(sc: &ScriptParam, vw: &mut ArViewDocInfo) -> Vec<Vec<String>> {
    //println!("DIAG DOC : {sc:?}");
    let mut rows = Vec::<Vec<String>>::new();
    //for dis in vw.edia.iter() {
    for dis in vw.adia.iter() {
        for di in dis.iter() {
            match di.dgtp {
                DiagramType::ViewRef => {
                    //docv = ar_gen_doc(docv, &di.eid, vw.es)?;
                }
                DiagramType::ElemRef => {
                    let mut xtp = if di.xtp.starts_with("archimate:") {
                        di.xtp[10..].to_string()
                    } else {
                        "".to_string()
                    };
                    if !di.prf.is_empty() {
                        xtp = di.prf.to_string();
                    }
                    if sc.xtype.is_empty() || sc.xtype.contains(&xtp) {
                        rows = elm_to_rows(rows, di.rc.clone(), vw);
                    }
                }
                DiagramType::NoteRef => {}
                DiagramType::None => {}
            }
        }
    }
    rows
}

pub fn ass_reorder(assv0: &mut [PeaAssVar], sc: &ScriptParam) {
    let mut done = false;
    if !sc.max2min.is_empty() {
        let ei = fld_2_var(sc.max2min[0].as_str());
        if let VarType::None = ei {
        } else {
            println!(">>>>>>>>>>>>>>>>>>>>> MAX2MIN ORDER ");
            assv0.sort_by(|b, a| {
                let a0 = a.v[ei.tousz()].v;
                let b0 = b.v[ei.tousz()].v;
                a0.partial_cmp(&b0).unwrap()
            });
            done = true;
        }
    } else if !sc.min2max.is_empty() {
        let ei = fld_2_var(sc.min2max[0].as_str());
        if let VarType::None = ei {
        } else {
            println!(">>>>>>>>>>>>>>>>>>>>> MIN2MAX ORDER ");
            assv0.sort_by(|a, b| {
                let a0 = a.v[ei.tousz()].v;
                let b0 = b.v[ei.tousz()].v;
                a0.partial_cmp(&b0).unwrap()
            });
            done = true;
        }
    }
    if !done {
        assv0.sort_by(|a, b| {
            let a0 = a.v[VarType::Uc1Rank.tousz()].v
                + a.v[VarType::Uc2Rank.tousz()].v
                + a.v[VarType::Uc3Rank.tousz()].v;
            let b0 = b.v[VarType::Uc1Rank.tousz()].v
                + b.v[VarType::Uc2Rank.tousz()].v
                + b.v[VarType::Uc3Rank.tousz()].v;
            a0.partial_cmp(&b0).unwrap()
        });
    }
}

pub fn tab_row_popu(sc: &mut ScriptParam, assv0: &[PeaAssVar], nmtp: &str) -> Vec<Vec<String>> {
    let dnm = crate::dcl::get_dirnm();
    let dum = vec![vec!["".to_string()]];
    let buf = std::fs::read(format!("{dnm}/000-subm.bin")).unwrap();

    //======== SUB INFO
    let Ok((subm, _)): Result<(HashMap<String, PeaSub>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };

    //======== AOJ INFO
    let buf = std::fs::read(format!("{dnm}/000-aojm.bin")).unwrap();
    //let Ok((aojm, _)): Result<(HashMap<String, AojInfo>, usize), _> =
    let Ok((aojm, _)): Result<(HashMap<String, BranchGIS>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };

    if !sc.yrs.is_empty() {
        let mut hds = Vec::<String>::new();
        for i in 0..(sc.fld.len() - 1) {
            let hd = if i < sc.hds.len() {
                sc.hds[i].to_string()
            } else {
                sc.fld[i].to_string()
            };
            hds.push(hd);
        }
        for y in &sc.yrs {
            hds.push(format!("ปี{}", y + 2026));
        }
        sc.hds = hds;
    }
    let mut vv = Vec::<Vec<String>>::new();
    let mut vsm = Vec::<f32>::new();
    for (i, a) in assv0.iter().enumerate() {
        if sc.lmt > 0 && i >= sc.lmt {
            break;
        }
        let mut v = Vec::<String>::new();
        v.push((i + 1).to_string());
        let nm = match nmtp {
            "P" => a.pvid.to_string(),
            "S" => {
                if let Some(psb) = subm.get(&a.sbid) {
                    format!("{} - {}", a.sbid, psb.name)
                } else {
                    a.sbid.to_string()
                }
            }
            "A" => {
                if let Some(ao) = aojm.get(&a.aoj) {
                    let anm = ao.name.clone().unwrap_or(String::new()).to_string();
                    format!("{} - {anm}", a.aoj)
                } else {
                    a.aoj.to_string()
                }
            }
            "T" => a.peano.to_string(),
            _ => "?".to_string(),
        };
        v.push(nm);
        //v.push(a.pvid.clone());
        let mut vas = Vec::<f32>::new();
        let mut tk = sc.fld.len();
        if !sc.yrs.is_empty() {
            tk -= 1;
        }
        //println!("======= TB FIELD : {:?}", sc.fld);
        for (i, (fd, pn)) in sc.fld.iter().zip(sc.pan.iter()).skip(2).enumerate() {
            if i >= tk - 2 {
                break;
            }
            let ii = fld_2_var(fd).tousz();
            let va = a.v[ii].v;
            let vp = val_2_form(va, pn.as_str());
            if sc.sum {
                vas.push(va);
            }
            v.push(vp);
        }
        if !sc.yrs.is_empty() {
            let ii = fld_2_var(&sc.fld[tk - 1]).tousz();
            let pn = sc.pan[sc.pan.len() - 1].to_string();
            for y in sc.yrs.iter() {
                if *y < a.vy[ii].len() {
                    let va = a.vy[ii][*y];
                    let vp = val_2_form(va, pn.as_str());
                    if sc.sum {
                        vas.push(va);
                    }
                    v.push(vp);
                }
            }
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
            let pi = if i + 2 >= sc.pan.len() {
                sc.pan.len() - 1
            } else {
                i + 2
            };
            let pn = sc.pan[pi].clone();
            let mut vp = val_2_form(*va, pn.as_str());
            if pn == "P" {
                vp = "".to_string();
            }
            v.push(vp);
        }
        vv.push(v);
    }
    vv
}

pub fn ass_var_prv(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    let dnm = crate::dcl::get_dirnm();
    let dum = vec![vec!["1.1".to_string()]];
    let buf = std::fs::read(format!("{dnm}/000-pvrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    ass_reorder(&mut assv0, sc);
    tab_row_popu(sc, &assv0, "P")
}

pub fn ass_var_sub(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    println!("=========== pvid: {}", sc.pvid);
    let dnm = crate::dcl::get_dirnm();
    let dum = vec![vec!["1.1".to_string(), "1.2".to_string()]];
    //println!("ass var prv sc:{sc:?}");
    let buf = std::fs::read(format!("{dnm}/000-sbrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    /*
    let buf = std::fs::read(format!("{dnm}/000-subm.bin")).unwrap();
    let Ok((subm, _)): Result<(HashMap<String, PeaSub>, usize), _> =
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
    tab_row_popu(sc, &assv0, "S")
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
    //println!("ass: {}", assv0.len());
    /*
    let mut vv = Vec::<Vec<String>>::new();
    let mut vsm = Vec::<f32>::new();
    for (i, a) in assv0.iter().enumerate() {
        if sc.lmt > 0 && i >= sc.lmt {
            break;
        }
        let mut v = Vec::<String>::new();
        v.push((i + 1).to_string());
        let sbnm = if let Some(psb) = subm.get(&a.sbid) {
            //format!("{} - {} - {}", a.sbid, psb.name, psb.prov)
            format!("{} - {}", a.sbid, psb.name)
        } else {
            a.sbid.to_string()
        };
        v.push(sbnm);
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

pub fn ass_var_sub_tr(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    //println!("=========== sbid: {}", sc.sbid);
    let dnm = crate::dcl::get_dirnm();
    let dum = vec![vec!["1.1".to_string(), "1.2".to_string()]];
    let Ok(buf) = std::fs::read(format!("{dnm}/{}-rw4.bin", sc.sbid)) else {
        return dum;
    };
    //println!("ass var prv sc:{sc:?}");
    //let buf = std::fs::read(format!("{dnm}/000-sbrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    if !sc.sbid.is_empty() {
        assv0 = assv0
            .into_iter()
            .filter(|a| a.sbid == sc.sbid && a.v[VarType::NoPeaTr.tousz()].v > 0.0)
            .collect::<_>();
    }
    ass_reorder(&mut assv0, sc);
    tab_row_popu(sc, &assv0, "T")
}

pub fn ass_var_fd_tr(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    //println!("=========== fdid: {}", sc.fdid);
    let dnm = crate::dcl::get_dirnm();
    let dum = vec![vec!["1.1".to_string(), "1.2".to_string()]];
    let Ok(buf) = std::fs::read(format!("{dnm}/{}-rw4.bin", &sc.fdid[..3])) else {
        println!(" >>>>>>>> NOFILE {}", &sc.fdid[..3]);
        return dum;
    };
    //println!("ass var prv sc:{sc:?}");
    //let buf = std::fs::read(format!("{dnm}/000-sbrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    if !sc.fdid.is_empty() {
        assv0 = assv0
            .into_iter()
            .filter(|a| {
                a.fdid == sc.fdid
                    && a.v[VarType::NoPeaTr.tousz()].v > 0.0
                    && a.v[VarType::AllNoMeterTr.tousz()].v > 0.0
            })
            .collect::<_>();
    }
    ass_reorder(&mut assv0, sc);
    tab_row_popu(sc, &assv0, "T")
}

pub fn ass_var_prv_yr(sc: &mut ScriptParam) -> Vec<Vec<String>> {
    let dum = vec![vec!["1.1".to_string()]];
    //println!("ass var prv sc:{sc:?}");
    let dnm = crate::dcl::get_dirnm();
    let buf = std::fs::read(format!("{dnm}/000-pvrw.bin")).unwrap();
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return dum;
    };
    ass_reorder(&mut assv0, sc);
    //println!("ass: {}", assv0.len());
    let mut ys = vec![0, 1, 2, 3, 12, 13, 14];
    if !sc.yrs.is_empty() {
        ys = sc.yrs.clone();
    }
    println!("years: {:?}", sc.yrs);
    let mut hds = Vec::<String>::new();
    for (i, h) in sc.hds.iter().enumerate() {
        if i > 2 {
            break;
        }
        hds.push(h.clone());
    }
    for i in &ys {
        hds.push(format!("{}", i + 2026));
    }
    sc.hds = hds;

    let mut vv = Vec::<Vec<String>>::new();
    let mut vsm = Vec::<f32>::new();
    let fd = sc.fld[2].clone();
    let pn = sc.pan[2].clone();
    for (i, a) in assv0.iter().enumerate() {
        if sc.lmt > 0 && i >= sc.lmt {
            break;
        }
        let mut v = Vec::<String>::new();
        v.push((i + 1).to_string());
        v.push(a.pvid.clone());
        let i = fld_2_var(&fd).tousz();
        let mut vas = Vec::<f32>::new();
        for y in &ys {
            if *y >= a.vy.len() {
                continue;
            }
            let va = a.vy[i][*y];
            if sc.sum {
                vas.push(va);
            }
            let vp = val_2_form(va, pn.as_str());
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
        for va in &vsm {
            let mut vp = val_2_form(*va, pn.as_str());
            if pn == "P" {
                vp = "".to_string();
            }
            v.push(vp);
        }
        vv.push(v);
    }
    vv
}

pub fn add_2_sum(mut vsm: Vec<f32>, vas: Vec<f32>) -> Vec<f32> {
    for (i, va) in vas.iter().enumerate() {
        if i >= vsm.len() {
            vsm.push(*va);
        } else {
            vsm[i] += *va;
        }
    }
    vsm
}

//======================================================
pub fn tb1(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &mut ArViewDocInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let mut sc = script_param(pws);

    // table title
    let lst = pws.last().unwrap();
    println!("tb1 {lst}");
    docv = sp(docv, vw)?;
    let lst = modi_tab(lst.as_str());
    let pa = Paragraph::new()
        .add_run(
            Run::new()
                .add_text(lst)
                .size(32)
                .fonts(RunFonts::new().cs(STD_FONT)),
        )
        .align(AlignmentType::Center);
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;

    // table func dispatcher
    let data = match &sc.cmd {
        None => data0(&sc),
        Some(c) => match c.as_str() {
            //"data0" => data0(&sc),
            "diag_to_rows" => diag_to_rows(&sc, vw),
            "ass_var_prv" => ass_var_prv(&mut sc),
            "ass_var_sub" => ass_var_sub(&mut sc),
            "ass_var_sub_tr" => ass_var_sub_tr(&mut sc),
            "ass_var_fd_tr" => ass_var_fd_tr(&mut sc),
            "ass_var_aoj_tr" => ass_var_aoj_tr(&mut sc),
            "ass_var_aoj" => ass_var_aoj(&mut sc),
            "ass_var_prv_yr" => ass_var_prv_yr(&mut sc),
            /*
            "DiagDoc" => diag_to_rows(&sc, vw),
            "AssVarPrv" => ass_var_prv(&sc),
            "AssVarSub" => ass_var_sub(&sc),
            "AssVarSubTr" => ass_var_sub_tr(&sc),
            "AssVarFdTr" => ass_var_fd_tr(&sc),
            "AssVarPrvYr" => ass_var_prv_yr(&mut sc),
            "AssVarAoj" => ass_var_aoj(&sc),
            "AssVarAojTr" => ass_var_aoj_tr(&sc),
            */
            _ => data0(&sc),
        },
    };

    if data.is_empty() {
        return Ok(docv);
    }

    let rw0 = &data[0];

    let mut rows = Vec::<TableRow>::new();
    //==== table header begin
    let mut hdrw = Vec::<TableCell>::new();
    //println!("header #2");
    let mut cwd = 100;
    for c in 0..rw0.len() {
        let mut rr = Run::new();
        let h = if c < sc.hds.len() {
            sc.hds[c].to_string()
        } else {
            if c < sc.fld.len() {
                sc.fld[c].to_string()
            } else {
                "-".to_string()
            }
        };
        rr = rr.add_text(&h);
        rr = rr.fonts(RunFonts::new().cs(STD_FONT));
        rr = rr.size(32);
        let mut pa = Paragraph::new();
        pa = pa.add_run(rr);
        pa = pa.align(AlignmentType::Center);
        let mut ce = TableCell::new();
        ce = ce.add_paragraph(pa);
        ce = ce.shading(Shading::new().fill(TABSHADE));
        if c < sc.wds.len() {
            cwd = sc.wds[c];
            //let w = sc.wds[c];
            //println!("  '{h}' - {w}");
        }
        ce = ce.width(cwd, WidthType::Dxa);
        hdrw.push(ce);
    }
    rows.push(TableRow::new(hdrw));
    //====  table header end

    for rw in data.iter() {
        let mut dtcs = Vec::<TableCell>::new();
        for (c, ce) in rw.iter().enumerate() {
            let mut rr = Run::new();
            rr = rr.add_text(ce);
            //rr = rr.add_text(format!("r:{i}.{j}"));
            rr = rr.fonts(RunFonts::new().cs(STD_FONT));
            rr = rr.size(32);
            let mut pa = Paragraph::new();
            pa = pa.add_run(rr);
            let a = if c < sc.als.len() {
                sc.als[c]
            } else {
                AlignmentType::Center
            };
            pa = pa.align(a);
            pa = pa.indent(Some(CELL_MARGIN), None, Some(CELL_MARGIN), None);
            let mut ce = TableCell::new();
            ce = ce.add_paragraph(pa);
            dtcs.push(ce);
        }
        rows.push(TableRow::new(dtcs));
    }
    let tb = Table::new(rows);
    let tb = tb.align(TableAlignmentType::Center);
    //let tb = tb.indent(DOC_TB_INDENT);
    docv = add_docxv(docv, DocxCompo::Table(tb), vw)?;

    let pa = Paragraph::new().add_run(Run::new());
    docv = add_docxv(docv, DocxCompo::Paragraph(pa), vw)?;
    Ok(docv)
}

pub fn ar_line_doc(
    mut docv: Vec<DocxCompo>,
    lines: &[String],
    vw: &mut ArViewDocInfo,
    isvw: bool,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let mut linev = Vec::<Vec<String>>::new();
    for ln in lines.iter() {
        let ln = ln.replace("$name", &vw.vnm);
        let pws = ar_cmd_split(ln.as_str());
        docv = match pws[0].as_str() {
            "h1" => hn(docv, &pws, 1, vw)?,
            "h2" => hn(docv, &pws, 2, vw)?,
            "h3" => hn(docv, &pws, 3, vw)?,
            "h4" => hn(docv, &pws, 4, vw)?,
            "h5" => hn(docv, &pws, 5, vw)?,
            "sp" => sp(docv, vw)?,
            "pg" => sp(docv, vw)?,
            "img1" => img1(docv, &pws, vw)?,
            "img2" => img2(docv, &pws, vw)?,
            "img3" => img3(docv, &pws, vw)?,
            "d1" => dn(docv, &pws, 1, vw)?,
            "d2" => dn(docv, &pws, 2, vw)?,
            "d3" => dn(docv, &pws, 3, vw)?,
            "d4" => dn(docv, &pws, 4, vw)?,
            "d5" => dn(docv, &pws, 5, vw)?,
            "doc1" => dn(docv, &pws, 1, vw)?,
            "doc2" => dn(docv, &pws, 2, vw)?,
            "doc3" => dn(docv, &pws, 3, vw)?,
            "t1" => tb1(docv, &pws, vw)?,
            "v1" if isvw => v1(docv, &pws, vw)?,
            "v2" if isvw => v2(docv, &pws, vw, false)?,
            "v3" if isvw => v2(docv, &pws, vw, true)?,
            "e1" if isvw => e1(docv, &pws, vw)?,
            "e2" if isvw => e2(docv, &pws, vw)?,
            "c1" => c1(docv, &pws, vw)?,
            "c2" => c2(docv, &pws, vw)?,
            "c3" => c3(docv, &pws, vw)?,
            _ => doc0(docv, &pws, vw)?,
        };
        linev.push(pws);
    }
    Ok(docv)
}

pub fn ar_view_elem(
    e: &std::cell::Ref<ArchElem>,
    es: &HashMap<String, Rc<RefCell<ArchElem>>>,
) -> (Vec<Diagram>, Vec<Diagram>, Vec<Diagram>) {
    let mut diavw = Vec::<Diagram>::new();
    let mut diaem = Vec::<Diagram>::new();
    let mut diaal = Vec::<Diagram>::new();
    println!("count elem: {}", e.child.len());
    for (i,ch) in e.child.iter().enumerate() {
        let rc = ch.clone();
        let ch = ch.borrow();
        let Some(tp) = ch.attr.get("xsi:type") else {
            continue;
        };
        let mut eid = "".to_string();
        let mut xtp = "".to_string();
        let mut prf = "".to_string();
        let mut vtp = "".to_string();
        let dgtp = match tp.as_str() {
            "archimate:DiagramModelReference" => {
                eid = ch.attr.get("model").unwrap_or(&eid).to_string();
                if let Some(el) = es.get(&eid) {
                    let el = el.borrow();
                    xtp = el.attr.get("xsi:type").unwrap_or(&xtp).to_string();
                    //println!("model view ref {xtp}");
                }
                DiagramType::ViewRef
            }
            "archimate:DiagramObject" => {
                eid = ch.attr.get("archimateElement").unwrap_or(&eid).to_string();
                if let Some(el) = es.get(&eid) {
                    let el = el.borrow();
                    let nm = el.attr.get("name").unwrap_or(&xtp).to_string();
                    let id = el.attr.get("id").unwrap_or(&xtp).to_string();
                    let cid= ch.attr.get("id").unwrap_or(&xtp).to_string();
                    println!("  {i}.elem name: {nm} id:{id} eid={eid} cid:{cid}");
                    xtp = el.attr.get("xsi:type").unwrap_or(&xtp).to_string();
                    vtp = el.attr.get("viewpoint").unwrap_or(&vtp).to_string();
                    let mut prfid = "".to_string();
                    prfid = el.attr.get("profiles").unwrap_or(&prfid).to_string();
                    //println!("diagram ref: {xtp}");
                    //println!("============ PROF ref: {prfid}");
                    if let Some(prof) = es.get(&prfid) {
                        let prof = prof.borrow();
                        let mut pnm = "".to_string();
                        pnm = prof.attr.get("name").unwrap_or(&pnm).to_string();
                        //println!("   >>>>>>>>======  profile name : {pnm}");
                        prf = pnm;
                    }
                }
                DiagramType::ElemRef
            }
            "archimate:Note" => DiagramType::NoteRef,
            d => {
                println!("Uncovered Diagram ..1: {d}");
                DiagramType::None
            }
        };
        if let DiagramType::None = dgtp {
            println!("ERROR DIAGRAM");
            continue;
        }
        let Ok(bnd) = get_elem_bounds(&ch) else {
            return (diavw, diaem, diaal);
        };
        let dia = Diagram {
            eid,
            dgtp,
            bnd,
            rc,
            xtp,
            prf,
            vtp,
        };
        match dia.dgtp {
            DiagramType::ViewRef => {
                diavw.push(dia.clone());
                diaal.push(dia);
            }
            DiagramType::ElemRef => {
                diaem.push(dia.clone());
                diaal.push(dia);
            }
            _ => {}
        }
    }
    (diavw, diaem, diaal)
}

pub fn ar_view_ids(
    mut vids: Vec<String>,
    vid: &str,
    es: &HashMap<String, Rc<RefCell<ArchElem>>>,
) -> Result<Vec<String>, Box<dyn Error>> {
    if let Some(e) = es.get(vid) {
        vids.push(vid.to_string());
        let e = e.borrow();
        let (diavw, _diaem, _diaal) = ar_view_elem(&e, es);
        for e in &diavw {
            vids = ar_view_ids(vids, &e.eid, es)?;
        }
        Ok(vids)
    } else {
        Err(format!("No eid {vid}").into())
    }
}

pub fn diag_rearange(diavw: &Vec<Diagram>) -> Vec<Vec<Diagram>> {
    let mut disv: Vec<Vec<_>> = vec![];
    if !diavw.is_empty() {
        let mut bndv = vec![diavw[0].bnd.clone()];
        disv = vec![vec![]];
        for di in diavw {
            let mut cn = 0;
            for (i, bnd) in bndv.iter_mut().enumerate() {
                if di.bnd.x0 <= bnd.x1 && di.bnd.x1 >= bnd.x0 {
                    bnd.x0 = di.bnd.x0.min(bnd.x0);
                    bnd.x1 = di.bnd.x1.max(bnd.x1);
                    cn += 1;
                    disv[i].push(di.clone());
                }
            }
            if cn == 0 {
                bndv.push(di.bnd.clone());
                disv.push(vec![di.clone()]);
            }
        }
    }
    disv.sort_by(|a, b| a[0].bnd.x0.cmp(&b[0].bnd.x0));
    //for (i, dis) in disv.iter_mut().enumerate() {
    for dis in disv.iter_mut() {
        //println!("== {i}-{}", dis.len());
        dis.sort_by(|a, b| a.bnd.y0.cmp(&b.bnd.y0));
    }
    //println!("bndv: {}", bndv.len());
    disv
}

pub fn ar_gen_doc(
    docv: Vec<DocxCompo>,
    vid: &str,
    es: &HashMap<String, Rc<RefCell<ArchElem>>>,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let mut docv = docv;
    println!("ar gen doc vid: {vid}");
    if let Some(e) = es.get(vid) {
        let e = e.borrow();

        let doc = get_elem_doc(&e);
        let lines = txt_lines(doc);
        let (diavw, diaem, diaal) = ar_view_elem(&e, es);
        let vnm = if let Some(n) = e.attr.get("name") {
            n.to_string()
        } else {
            "?".to_string()
        };
        println!("name: {vnm}");

        let nms = e.names.to_string();
        let edia = diag_rearange(&diaem);
        let vdia = diag_rearange(&diavw);
        let adia = diag_rearange(&diaal);
        println!("edia: {} vdia: {}", edia.len(), vdia.len());
        let mut vw = ArViewDocInfo {
            vid,
            vnm,
            nms,
            es,
            vdia: &vdia,
            edia: &edia,
            adia: &adia,
        };
        let l1 = docv.len();
        docv = ar_line_doc(docv, &lines, &mut vw, true)?;
        let l2 = docv.len();
        if l2 > l1 {
            docv = end_ref(docv, &vw)?;
        }
        Ok(docv)
    } else {
        Err(format!("No eid {vid}").into())
    }
}

pub fn get_elem_bounds(el: &std::cell::Ref<ArchElem>) -> Result<Bound, Box<dyn Error>> {
    for ch in &el.child {
        let ch = ch.borrow();
        if ch.elem == "bounds" {
            let x0 = ch.attr.get("x").unwrap().parse::<i32>()?;
            let y0 = ch.attr.get("y").unwrap().parse::<i32>()?;
            let x1 = x0 + ch.attr.get("width").unwrap().parse::<i32>()?;
            let y1 = y0 + ch.attr.get("height").unwrap().parse::<i32>()?;
            return Ok(Bound { x0, y0, x1, y1 });
        }
    }
    Err("Error".into())
}

#[derive(Encode, Decode, Debug, Clone, Default)]
pub struct ArchiState {
    pub hdno: [usize; 5],
    pub hdfg: [bool; 5],
    pub figno: usize,
    pub tabno: usize,
}

pub static ARCHISTATE: LazyLock<Mutex<ArchiState>> = LazyLock::new(|| {
    Mutex::new(ArchiState {
        hdno: [0; 5],
        hdfg: [true, true, true, false, false],
        ..Default::default()
    })
});


pub fn modi_head(hds: &str, hi: usize) -> String {
    let mut arst = ARCHISTATE.lock().unwrap();
    let mut hdn = "".to_string();
    for ii in 0..5 {
        if ii == hi - 1 {
            arst.hdno[ii] += 1;
        }
        if ii > hi - 1 {
            arst.hdno[ii] = 0;
        }
        if ii < hi {
            if !hdn.is_empty() {
                hdn = format!("{hdn}.");
            }
            hdn = format!("{hdn}{}", arst.hdno[ii]);
        }
    }
    if arst.hdfg[hi - 1] {
        format!("{hdn}. {hds}")
    } else {
        hds.to_string()
    }
}

pub fn modi_fig(hds: &str) -> String {
    let mut arst = ARCHISTATE.lock().unwrap();
    arst.figno += 1;
    format!("ภาพที่ {}. {hds}", arst.figno)
}

pub fn modi_tab(hds: &str) -> String {
    let mut arst = ARCHISTATE.lock().unwrap();
    arst.tabno += 1;
    format!("ตารางที่ {}. {hds}", arst.tabno)
}

#[derive(Debug)]
pub enum DocxCompo {
    Paragraph(Paragraph),
    Style(Style),
    Header(Header),
    Footer(Footer),
    Table(Table),
    ToC(TableOfContents),
}

pub const DOCX_PNTS: i32 = 9525;

pub fn write_docx(
    vdoc: Vec<DocxCompo>,
    pin: &str,
    pout: &str,
    tdir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut docx = Docx::new();
    let fst_hd = Header::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(" ")));
    let mag = PageMargin {
        top: 1000,
        left: 800,
        bottom: 1000,
        right: 1000,
        header: 500,
        footer: 500,
        gutter: 0,
    };
    docx = docx.page_margin(mag);
    for dc in vdoc {
        match dc {
            DocxCompo::Header(h) => {
                docx = docx.header(h);
            }
            DocxCompo::Footer(f) => {
                docx = docx.footer(f);
            }
            DocxCompo::Style(s) => {
                docx = docx.add_style(s);
            }
            DocxCompo::Paragraph(p) => {
                docx = docx.add_paragraph(p);
            }
            DocxCompo::Table(p) => {
                docx = docx.add_table(p);
            }
            DocxCompo::ToC(t) => {
                docx = docx.add_table_of_contents(t);
            }
        }
    }
    docx = docx.first_header(fst_hd);

    let pin = std::path::Path::new(pin);
    let pin1 = std::path::Path::new(pin);
    let pin2 = std::fs::File::create(pin1).unwrap();
    let _ = docx.build().pack(pin2);

    let pout = std::path::Path::new(pout);
    //println!("pin:{pin:?} pout:{pout:?}");

    let fout = std::fs::File::create(pout).unwrap();
    let mut zout = zip::ZipWriter::new(fout);
    let file = fs::File::open(pin).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        if file.is_dir() {
            let dir = format!("{}", outpath.display());
            let dir0 = format!("{}/{}", tdir, outpath.display());
            //println!("dir0:{dir0}");
            fs::create_dir_all(&dir0).unwrap();
            zout.add_directory(&dir, SimpleFileOptions::default())
                .expect("?");
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    //fs::create_dir_all(p).unwrap();
                }
            }
            let fnm = format!("{}", outpath.display());
            let fnm0 = format!("{}/{}", tdir, outpath.display());

            let mut outfile = fs::File::create(&fnm0).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();

            let mut f = std::fs::File::open(&fnm0).expect("no file found");
            let metadata = std::fs::metadata(&fnm0).expect("unable to read metadata");
            let mut buff = vec![0; metadata.len() as usize];
            f.read_exact(&mut buff).expect("buffer overflow");
            //if fnm == "word/document.xml" {
            if ["word/document.xml", "word/header1", "word/footer1"].contains(&fnm.as_str()) {
                let buf2 = String::from_utf8(buff.clone()).unwrap();
                let buf2 = buf2.replace("<w:rFonts", "<w:cs/><w:rFonts");
                /*
                let buf2 = buf2.replace("allowOverlap=\"0\"", "allowOverlap=\"1\"");
                let buf2 = buf2.replace("behindDoc=\"0\"", "behindDoc=\"1\"");
                let buf2 = buf2.replace(
                    "<wp:positionH relativeFrom=\"margin\"><wp:posOffset>0</wp:posOffset>",
                    "<wp:positionH relativeFrom=\"margin\"><wp:posOffset>-3175</wp:posOffset>",
                );
                let buf2 = buf2.replace(
                    "<wp:positionV relativeFrom=\"margin\"><wp:posOffset>0</wp:posOffset>",
                    "<wp:positionV relativeFrom=\"margin\"><wp:posOffset>3175</wp:posOffset>",
                );
                let buf2 = buf2.replace(
                    "<w:rPr><w:sz w:val=\"60\" /><w:szCs w:val=\"60\" /><w:cs/><w:rFonts w:cs=\"TH Sarabun New\" /></w:rPr><w:t xml:space=\"preserve\">",
                    "<w:rPr><w:sz w:val=\"60\" /><w:szCs w:val=\"60\" /><w:cs/><w:rFonts w:cs=\"TH Sarabun New\" /></w:rPr><w:t>"
                );
                */
                //println!("fnm:{fnm} 1:{} 2:{}", buff.len(), buf2.len());
                buff = buf2.as_bytes().to_vec();
            }
            let options = SimpleFileOptions::default();
            zout.start_file(fnm, options).expect("?");
            zout.write_all(&buff).expect("?");
        }
    }
    zout.finish().expect("?");
    Ok(())
}

pub fn create_docx_def0() -> Vec<DocxCompo> {
    let mut docv = Vec::<DocxCompo>::new();
    let style1 = Style::new("Heading1", StyleType::Paragraph).name("Heading 1");
    let style2 = Style::new("Heading2", StyleType::Paragraph).name("Heading 2");
    let style3 = Style::new("Heading3", StyleType::Paragraph).name("Heading 3");
    let header = Header::new().add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text("=== PEA Smart Grid Implementation Plan ===")
                    .fonts(RunFonts::new().cs(STD_FONT))
                    .size(14),
            )
            .align(AlignmentType::Center),
    );
    let footer = Footer::new().add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text("---")
                    .fonts(RunFonts::new().cs(STD_FONT))
                    .size(14),
            )
            .align(AlignmentType::Center),
    );
    let footer = footer.add_paragraph(
        Paragraph::new()
            .add_page_num(PageNum::new())
            .align(AlignmentType::Right),
    );

    docv.push(DocxCompo::Style(style1));
    docv.push(DocxCompo::Style(style2));
    docv.push(DocxCompo::Style(style3));
    docv.push(DocxCompo::Header(header));
    docv.push(DocxCompo::Footer(footer));
    docv
}

pub fn create_docx_def() -> Vec<DocxCompo> {
    let mut docv = Vec::<DocxCompo>::new();
    let style1 = Style::new("Heading1", StyleType::Paragraph).name("Heading 1");
    let style2 = Style::new("Heading2", StyleType::Paragraph).name("Heading 2");
    let style3 = Style::new("Heading3", StyleType::Paragraph).name("Heading 3");
    let header = Header::new().add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text("=== PEA Smart Grid Implementation Plan ===")
                    .fonts(RunFonts::new().cs(STD_FONT))
                    .size(14),
            )
            .align(AlignmentType::Center),
    );
    let footer = Footer::new().add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text("---")
                    .fonts(RunFonts::new().cs(STD_FONT))
                    .size(14),
            )
            .align(AlignmentType::Center),
    );
    let footer = footer.add_paragraph(
        Paragraph::new()
            .add_page_num(PageNum::new())
            .align(AlignmentType::Right),
    );
    let runp = RunProperty::new()
        .fonts(RunFonts::new().cs(STD_FONT))
        .size(32);
    let papr = ParagraphProperty::new().run_property(runp);
    let papr = papr.page_break_before(true);
    let toc = TableOfContents::new();
    let toc = toc.heading_styles_range(1, 2);
    //let toc = toc.heading_styles_range(1, 3);
    let toc = toc.paragraph_property(papr);
    let toc = toc.auto();
    //let cvimg = format!("{VW_IMG_PATH}cover1.png");
    //let cvimg = format!("{VW_IMG_PATH}cover2.png");
    let dnm = crate::dcl::get_dirnm();
    let cvimg = format!("{dnm}/eaviews/cover2.png");
    let mut img = std::fs::File::open(&cvimg).unwrap();
    let mut buf = Vec::new();
    let _ = img.read_to_end(&mut buf).unwrap();
    let pic = Pic::new(&buf)
        .size(700 * 9525, 1000 * 9525)
        .floating()
        .offset_x(0)
        .offset_y(0);

    let mut vtt = vec![];
    for _i in 0..13 {
        let tt = Paragraph::new().add_run(
            Run::new()
                .add_text(" ")
                .fonts(RunFonts::new().cs(STD_FONT))
                .size(60),
        );
        vtt.push(tt);
    }

    let tt = Paragraph::new().add_run(
        Run::new()
            .add_text("รายงานผลการวิเคราะห์ทางการเงินและเศรษฐศาสตร์")
            .fonts(RunFonts::new().cs(STD_FONT))
            .size(60),
    );
    vtt.push(tt);
    let tt = Paragraph::new().add_run(
        Run::new()
            .add_text("Financial and Economic Analysis Report")
            .fonts(RunFonts::new().cs(STD_FONT))
            .size(60),
    );
    vtt.push(tt);
    let tt = Paragraph::new().add_run(
        Run::new()
            .add_text("Smart Grid Implementation Plan")
            .fonts(RunFonts::new().cs(STD_FONT))
            .size(60),
    );
    vtt.push(tt);

    let pimg = Paragraph::new().add_run(Run::new().add_image(pic));

    let ptoc = Paragraph::new()
        .add_run(
            Run::new()
                .add_text("สารบัญ")
                .fonts(RunFonts::new().cs(STD_FONT))
                .size(40),
        )
        .page_break_before(true)
        .align(AlignmentType::Center);

    docv.push(DocxCompo::Style(style1));
    docv.push(DocxCompo::Style(style2));
    docv.push(DocxCompo::Style(style3));
    //docv.push(DocxCompo::Paragraph(pitt));
    for tt in vtt {
        docv.push(DocxCompo::Paragraph(tt));
    }
    docv.push(DocxCompo::Paragraph(pimg));
    docv.push(DocxCompo::Paragraph(ptoc));
    docv.push(DocxCompo::ToC(toc));
    docv.push(DocxCompo::Header(header));
    docv.push(DocxCompo::Footer(footer));
    docv
}


