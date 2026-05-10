//use phf_macros::phf_map;
//use crate::utl2::attr_map;
use chrono::Local;
use axum::routing::get;
use quick_xml::events::Event;
use quick_xml::reader::Reader as XmlReader;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::rc::Rc;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use std::io::{Read, Write};
use zip::write::SimpleFileOptions;
use strum_macros::EnumString;
use std::str::FromStr;
use crate::asm::ASM;
use std::borrow::Cow;
use crate::img::fda01::get_img;
use crate::dcl::BranchGIS;
use crate::dcl::PeaSub;
use crate::dcl::Pan;
use crate::dcl::PeaAssVar;
use crate::dcl::VarType;
use rust_xlsxwriter::{Workbook,Format,FormatAlign,FormatBorder,Color};
//use crate::utl5::DATA_FLDS;
//use crate::dcl::FIR_FLDS;

use crate::dcl::FIR_FLDS;
use crate::p08::ld_sub_info;
use crate::utl7::get_brn_map;

pub const ARCHI_INPUT:&str = "/sto/archi/PEA-SmartGrid.archimate";
pub const ARCH_OUTDIR: &str = "arch-data";
pub const STD_FONT: &str = "TH Sarabun New";
pub const ARCH_GENDIR: &str = "arch-gen";
pub const DOCX_PNTS: i32 = 9525;
pub const DOC_TB_INDENT: i32 = 500;
pub const TABSHADE: &str = "#ecffe6";
pub const CELL_MARGIN: i32 = 100;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ElemInfo {
    pub pa: Option<Rc<RefCell<XmlElem>>>,
    pub chd: Vec<Rc<RefCell<XmlElem>>>,
    pub attr: HashMap<String, String>,
    pub ex: Box<ElemInfoEx>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SourceConnection {
    pub id: String,
    pub xsitp: ArchiType,
    pub src_id: String,
    pub rel_id: String,
    pub trg_id: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ElemInfoEx {
    pub name: String,
    pub id: String,
    pub xsitp: ArchiType,
    pub tp: String,
    pub doc: String,
    pub prop: Vec<(String,String)>,
    pub bnd: Bounds,
    pub elm_id: String,
    pub img_pt: String,
    pub trgs: Vec<String>,
    pub mod_id: String,
    pub prf_id: String,
    pub src_cons: Vec<SourceConnection>,
    pub boxes: Vec<Vec<Rc<RefCell<XmlElem>>>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub height: i32,
    pub width: i32,
}

#[derive(Debug, Clone, Default, EnumIter, PartialEq, EnumString)]
pub enum XmlElem {
    #[default]
    None,
    Elem {
        tg: ArchiTag,
        em: ElemInfo,
    },
    TextElem {
        tx: String,
    },
    GenRef {
        tx: String,
    },
}

#[derive(Debug, Clone, Default, EnumIter, PartialEq, EnumString)]
#[allow(non_camel_case_types)]
pub enum ArchiProfile {
    #[default]
    None,
    BusinessActor_Swimlane,
    Principl_eReport,
    Artifact_AssumNum,
    BusinessObject_Assum,
    Artifact_Report,
    Artifact_AssumText,
    Artifact_AssumJson,
    Artifact_AssumToml,
    Principle_Report,
    BusinessObject_RequiredAssumption,
}

#[derive(Debug, Clone, Default, EnumIter, PartialEq, EnumString)]
#[allow(non_camel_case_types)]
pub enum ArchiTag {
    #[default]
    none,
    model,
    folder,
    profile,
    element,
    property,
    bounds,
    sourceConnection,
    bendpoint,
    feature,
    child,
    content,
    hintContent,
    documentation,
}

#[derive(Clone,Default)]
pub struct Assumption {
    pub nm_assu: HashMap<String,NumValEnum>,
    pub v_assum: Vec<NumValEnum>,
    pub v_asm_a: Vec<usize>,
    pub v_asm_r: Vec<f32>,
    pub v_asm_i: Vec<i32>,
    pub v_asm_s: Vec<String>,
    pub v_asm_u: Vec<usize>,
}

impl Assumption {
    pub fn ve(&self, asm: ASM) -> Result<NumValEnum, Box<dyn Error>> {
        Ok(self.v_assum[asm as usize].clone())
    }
    pub fn vv(&self, asm: ASM) -> Result<f32, Box<dyn Error>> {
        Ok(self.v_asm_r[asm as usize])
    }
    pub fn v(&self, asm: ASM) -> f32 {
        self.v_asm_r[asm as usize]
    }
    pub fn u(&self, asm: ASM) -> usize {
        self.v_asm_i[asm as usize] as usize
    }
    pub fn t(&self, asm: ASM) -> String {
        self.v_asm_s[asm as usize].clone()
    }
}

#[derive(Clone)]
pub struct ArchiInfo {
    pub view: String,
    pub e_view: Option<Rc<RefCell<XmlElem>>>,
    pub e_root: Option<Rc<RefCell<XmlElem>>>,
    pub id_elem: HashMap<String,Rc<RefCell<XmlElem>>>,
    pub nm_view: HashMap<String,Rc<RefCell<XmlElem>>>,
    pub id_prof: HashMap<String,Rc<RefCell<XmlElem>>>,
    pub ass: Assumption,
}

impl ArchiInfo {
    pub fn assumption(&self) -> Assumption {
        self.ass.clone()
            /*
        Assumption {
            nm_assu: self.nm_assu.clone(),
            v_asm_r: self.v_asm_r.clone(),
            v_asm_i: self.v_asm_i.clone(),
            v_asm_s: self.v_asm_s.clone(),
        }
            */
    }
    pub fn new(vn: &str) -> ArchiInfo {
        let view = vn.to_string();
        let id_elem = HashMap::<String,Rc<RefCell<XmlElem>>>::new();
        let id_prof = HashMap::<String,Rc<RefCell<XmlElem>>>::new();
        let nm_view = HashMap::<String,Rc<RefCell<XmlElem>>>::new();
        ArchiInfo {
            view,
            id_elem,
            id_prof,
            nm_view,
            e_view: None,
            e_root: None,
            ass: Assumption::default(),
        }
    }
    /*
    pub fn init_assum(&mut self) {
        let mut cn = 0;
        let mut s2n = HashMap::<String, usize>::new();
        for a in ASM::iter() {
            cn += 1;
            let an = format!("{a:?}");
            let ai = a as usize;
            s2n.insert(an.to_string(), ai);
            //println!("###0 i:{ai} n:{an}");
        }
        println!("asm.{}", self.nm_assu.len());
        self.v_asm_r = vec![0f32; cn];
        self.v_asm_i = vec![0i32; cn];
        self.v_asm_s = vec!["".to_string(); cn];
        for (k, v) in self.nm_assu.iter() {
            let kk = k.to_string();
            //println!("k:{k} v:{v:?}");
            let Some(ai) = s2n.get(&kk) else {
                continue;
            };
            //println!("{k} -> {v:?}");
            match v {
                NumValEnum::Real(v) => {
                    self.v_asm_r[*ai] = *v;
                }
                NumValEnum::Int(v) => {
                    self.v_asm_i[*ai] = *v;
                }
                NumValEnum::Text(v) => {
                    self.v_asm_s[*ai] = v.to_string();
                }
                _ => {}
            }
        }
    }
    pub fn ve(&self, asm: ASM) -> Result<NumValEnum, Box<dyn Error>> {
        let asss = format!("{asm:?}");
        if let Some(ass) = self.nm_assu.get(&asss) {
            return Ok(ass.clone());
        }
        Err("No Assumption - {ss:?}".into())
    }
    pub fn vv(&self, asm: ASM) -> Result<f32, Box<dyn Error>> {
        let asss = format!("{asm:?}");
        if let Some(ass) = self.nm_assu.get(&asss) && let NumValEnum::Real(a) = ass {
            return Ok(*a);
        }
        Err("No Assumption - {ss:?}".into())
    }
    pub fn v(&self, asm: ASM) -> f32 {
        self.v_asm_r[asm as usize]
    }
    pub fn u(&self, asm: ASM) -> usize {
        self.v_asm_i[asm as usize] as usize
    }
    pub fn t(&self, asm: ASM) -> String {
        self.v_asm_s[asm as usize].to_string()
    }
    */
}

#[derive(Debug, Clone, Default, EnumIter, PartialEq, EnumString)]
pub enum ArchiType {
    #[default]
    None,
    AccessRelationship,
    AggregationRelationship,
    ApplicationComponent,
    ApplicationEvent,
    ApplicationFunction,
    ApplicationInterface,
    ApplicationProcess,
    ApplicationService,
    ArchimateDiagramModel,
    Artifact,
    Assessment,
    AssignmentRelationship,
    AssociationRelationship,
    BusinessActor,
    BusinessCollaboration,
    BusinessEvent,
    BusinessFunction,
    BusinessInterface,
    BusinessObject,
    BusinessProcess,
    BusinessRole,
    BusinessService,
    Capability,
    CommunicationNetwork,
    CompositionRelationship,
    Constraint,
    Contract,
    CourseOfAction,
    DataObject,
    Deliverable,
    Device,
    DiagramModelReference,
    DiagramObject,
    DistributionNetwork,
    Driver,
    Equipment,
    Facility,
    FlowRelationship,
    Gap,
    Goal,
    Group,
    Grouping,
    ImplementationEvent,
    InfluenceRelationship,
    Junction,
    Location,
    Meaning,
    Node,
    Note,
    Outcome,
    Path,
    Plateau,
    Principle,
    Product,
    RealizationRelationship,
    Representation,
    Requirement,
    Resource,
    ServingRelationship,
    SpecializationRelationship,
    Stakeholder,
    SystemSoftware,
    TechnologyFunction,
    TechnologyInterface,
    TechnologyService,
    TriggeringRelationship,
    Value,
    ValueStream,
    WorkPackage,
    CanvasModel,
    CanvasModelBlock,
    CanvasModelImage,
    CanvasModelSticky,
}

//use crate::utl4::Archi;
use crate::asm::ASM::OUTDIR;
use crate::dcl::set_dirnm;
use crate::stg1::stage_01;
use crate::sty2::stage_02_1;
//use crate::sty2::stage_02_b;
use crate::sty2::stage_02_c;
use crate::sty2::stage_02_d;
use docx_rs::*;

#[derive(Debug)]
pub enum DocxCompo {
    //Paragraph(Paragraph),
    ParagEnum(Paragraph),
    StyleEnum(Box<Style>),
    HeaderEnum(Header),
    FooterEnum(Footer),
    TableEnum(Table),
    ToCEnum(TableOfContents),
}
//use crate::utl4::get_elem_doc;

pub fn ar_gen_doc(
    docv: Vec<DocxCompo>,
    arif: &ArchiInfo,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let mut docv = docv;
    let Some(ref evw) = arif.e_view else {
        return Err("No View Error #1".into());
    };
    /*
    {
        let mut evwa = evw.borrow_mut();
        if let XmlElem::Elem{tg:ArchiTag::element,em:em1} = &mut *evwa  {
            let dias = get_diag_in_view(evw.clone())?;
            println!("view dias: {}", dias.len());
            em1.ex.boxes = diag_rearange(&dias);
        }
    }
    */
    //let evw0 = evw.borrow();
    /*
    let XmlElem::Elem{tg:ArchiTag::element,em:em1} = &*evw.borrow() else {
        return Err("No View Error #2".into());
    };
    let doc = em1.ex.doc.clone();
    println!("view doc: {}", em1.ex.doc);
    let lines = txt_lines(doc);
    println!("lines: {}", lines.len());
    let vw = evw.clone();
    let dias = get_diag_in_view(vw)?;
    println!("dias: {}", dias.len());
    */
    let l1 = docv.len();
    docv = ar_line_doc(docv, evw, arif, true)?;
    let l2 = docv.len();
    if l2 > l1 {
        docv = end_ref(docv, evw)?;
    }
    //let doc = get_elem_doc(&e);
    /*

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
    */
    Ok(docv)
}
use image::ImageReader;

pub fn ar_line_doc(
    mut docv: Vec<DocxCompo>,
    evw: &Rc<RefCell<XmlElem>>,
    _arif: &ArchiInfo,
    isvw: bool,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    //if isvw {
    {
        let dias = get_diag_in_view(evw.clone())?;
        let mut evwa = evw.borrow_mut();
        if let XmlElem::Elem{tg:ArchiTag::element,em:em1} = &mut *evwa  {
            println!("view dias: {}", dias.len());
            em1.ex.boxes = diag_rearange(&dias);
        }
    }
    let XmlElem::Elem{tg:ArchiTag::element,em:em1} = & *evw.borrow() else {
        return Err("No View Error #2".into());
    };
    let doc = em1.ex.doc.clone();
    let vnm = em1.ex.name.clone();
    println!("view doc: {}", em1.ex.doc);
    let lines = txt_lines(doc);
    println!("lines: {}", lines.len());
    let mut linev = Vec::<Vec<String>>::new();
    for ln in lines.iter() {
        let ln = ln.replace("$name", &vnm);
        let pws = ar_cmd_split(ln.as_str());
        docv = match pws[0].as_str() {
            "h1" => hn(docv, &pws, 1, evw)?,
            "h2" => hn(docv, &pws, 2, evw)?,
            "h3" => hn(docv, &pws, 3, evw)?,
            "h4" => hn(docv, &pws, 4, evw)?,
            "h5" => hn(docv, &pws, 5, evw)?,
            "sp" => sp(docv, evw)?,
            "pg" => sp(docv, evw)?,
            "img1" => img1(docv, &pws, evw)?,
            "img2" => img2(docv, &pws, evw)?,
            "img3" => img3(docv, &pws, evw)?,
            "d1" => dn(docv, &pws, 1, evw)?,
            "d2" => dn(docv, &pws, 2, evw)?,
            "d3" => dn(docv, &pws, 3, evw)?,
            "d4" => dn(docv, &pws, 4, evw)?,
            "d5" => dn(docv, &pws, 5, evw)?,
            "doc1" => dn(docv, &pws, 1, evw)?,
            "doc2" => dn(docv, &pws, 2, evw)?,
            "doc3" => dn(docv, &pws, 3, evw)?,
            "t1" => tb1(docv, &pws, evw)?,
            "v1" if isvw => v1(docv, &pws, evw)?,
            "v2" if isvw => v2(docv, &pws, evw, false)?,
            "v3" if isvw => v2(docv, &pws, evw, true)?,
            "e1" if isvw => e1(docv, &pws, evw)?,
            "e2" if isvw => e2(docv, &pws, evw)?,
            /*

            "c1" => c1(docv, &pws, vw)?,
            "c2" => c2(docv, &pws, vw)?,
            "c3" => c3(docv, &pws, vw)?,
    */
            _ => doc0(docv, &pws, evw)?,
        };
        linev.push(pws);
    }
    Ok(docv)
}

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
            DocxCompo::HeaderEnum(h) => {
                docx = docx.header(h);
            }
            DocxCompo::FooterEnum(f) => {
                docx = docx.footer(f);
            }
            DocxCompo::StyleEnum(s) => {
                docx = docx.add_style(*s);
            }
            DocxCompo::ParagEnum(p) => {
                docx = docx.add_paragraph(p);
            }
            DocxCompo::TableEnum(p) => {
                docx = docx.add_table(p);
            }
            DocxCompo::ToCEnum(t) => {
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

pub fn diag_rearange(diavw: &[Rc<RefCell<XmlElem>>]) -> Vec<Vec<Rc<RefCell<XmlElem>>>> {
    let mut disv: Vec<Vec<_>> = vec![];
    if diavw.is_empty() {
        return disv;
    }
    let mut elemv: Vec<Vec<_>> = vec![vec![]];
    let mut boxes = Vec::<(Bounds,usize)>::new();
    for (i,re) in diavw.iter().enumerate() {
        let re0 = re.borrow();
        if let XmlElem::Elem{tg:ArchiTag::child, em:em1} = &*re0 {
            boxes.push((em1.ex.bnd.clone(),i));
        }
    }
    println!("all boxes: {}", boxes.len());
    let mut bndv = vec![boxes[0].clone()];
    for di in boxes.iter() {
        let mut cn = 0;
        for (i,bnd) in bndv.iter_mut().enumerate() {
            if di.0.x<= (bnd.0.x+bnd.0.width) && (di.0.x+di.0.width) >= bnd.0.x {
                bnd.0.x = di.0.x.min(bnd.0.x);
                let xri = (di.0.x+di.0.width).max(bnd.0.x+bnd.0.width);
                bnd.0.width = xri - bnd.0.x;
                cn += 1;
                elemv[i].push(di.clone());
            }
        }
        if cn==0 {
            bndv.push(di.clone());
            elemv.push(vec![di.clone()]);
        }
    }
    elemv.sort_by(|a, b| a[0].0.x.cmp(&b[0].0.x));
    for dis in elemv.iter_mut() {
        dis.sort_by(|a,b| a.0.y.cmp(&b.0.y));
        let mut disa = vec![];
        for (_b,i) in dis.iter() {
            disa.push(diavw[*i].clone());
        }
        disv.push(disa);
    }
    /*
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
    disv.sort_by(|a, b| a[0].bnd.x0.cmp(&b[0].bnd.x0));
    //for (i, dis) in disv.iter_mut().enumerate() {
    for dis in disv.iter_mut() {
        //println!("== {i}-{}", dis.len());
        dis.sort_by(|a, b| a.bnd.y0.cmp(&b.bnd.y0));
    }
    */
    //println!("bndv: {}", bndv.len());
    disv
}

pub fn get_elem_name(el: &Rc<RefCell<XmlElem>>) -> String {
    if let XmlElem::Elem{em:em1,..} = &*el.borrow() { em1.ex.name.clone() } else { "".to_string() }
}

pub fn get_elem_id(el: &Rc<RefCell<XmlElem>>) -> String {
    if let XmlElem::Elem{em:em1,..} = &*el.borrow() { em1.ex.id.clone() } else { "".to_string() }
}

pub fn get_elem_doc(el: &Rc<RefCell<XmlElem>>) -> String {
    if let XmlElem::Elem{em:em1,..} = &*el.borrow() { em1.ex.doc.clone() } else { "".to_string() }
}

pub fn get_elem_boxes(el: &Rc<RefCell<XmlElem>>) -> Vec<Vec<Rc<RefCell<XmlElem>>>> {
    if let XmlElem::Elem{em:em1,..} = &*el.borrow() { em1.ex.boxes.clone() } else { Vec::<_>::new() }
}

pub fn get_ref_text(vw: &Rc<RefCell<XmlElem>>) -> String {
    let mut tx = String::new();
    let mut ems = vec![vw.clone()];
    while let Some(em) = ems.pop() {
        let ema = em.borrow();
        match &*ema {
            XmlElem::Elem{tg:ArchiTag::folder,em:em1} |
            XmlElem::Elem{tg:ArchiTag::element,em:em1} => {
                let nm = em1.ex.name.clone();
                tx = format!("/{nm}{tx}");
                if let Some(pa) = &em1.pa {
                    ems.push(pa.clone());
                }
            }
            _=>{}
        }
    }
    tx
}

pub fn end_ref(docv: Vec<DocxCompo>, vw: &Rc<RefCell<XmlElem>>) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let mut ii = 0;
    for (i, doc) in docv.iter().enumerate() {
        if let DocxCompo::ParagEnum(_) = doc {
            ii = i;
        }
    }
    let mut vdoc = Vec::<DocxCompo>::new();
    for (i0, doc) in docv.into_iter().enumerate() {
        let doc = if i0 == ii {
            if let DocxCompo::ParagEnum(mut par) = doc {
                let v = get_ref_text(vw);
                println!("REF: {v}");
                //let v = &vw.nms[6..];
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
                DocxCompo::ParagEnum(par)
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


//======================================================
pub fn hn(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    n: i32,
    vw: &Rc<RefCell<XmlElem>>,
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
    docv = add_docxv(docv, DocxCompo::ParagEnum(h1), vw)?;
    println!("hn {n} - {lst}");
    Ok(docv)
}

//======================================================
pub fn dn(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    n: i32,
    vw: &Rc<RefCell<XmlElem>>,
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
    docv = add_docxv(docv, DocxCompo::ParagEnum(pa), vw)?;
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

pub fn diag_to_rows(_sc: &ScriptParam, vw: &Rc<RefCell<XmlElem>>) -> Vec<Vec<String>> {
    //println!("DIAG DOC : {sc:?}");
    let mut rows = Vec::<Vec<String>>::new();
    //for dis in vw.edia.iter() {
    let boxes = get_elem_boxes(vw);
    for dis in boxes.iter() {
        for di in dis.iter() {
            let die = di.borrow();
            if let XmlElem::Elem{tg:ArchiTag::child,em:em1} = &*die {
                if !em1.ex.mod_id.is_empty() {
                } else if !em1.ex.elm_id.is_empty() {
                    rows = elm_to_rows(rows, di.clone(), vw);
                }
            }
            /*
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
            */
        }
    }
    rows
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
    rc: Rc<RefCell<XmlElem>>,
    _vw: &Rc<RefCell<XmlElem>>,
) -> Vec<Vec<String>> {
    let name = get_elem_name(&rc);
    let doc = get_elem_doc(&rc);
    let no = format!("{}", rows.len());
    if !name.is_empty() && !doc.is_empty() {
        rows.push(vec![no, name, doc]);
    }
    /*
    
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
    */
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
    vw: &Rc<RefCell<XmlElem>>,
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
    docv = add_docxv(docv, DocxCompo::ParagEnum(pa), vw)?;

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
            "ass_var_prv_yr" => ass_var_prv_yr(&mut sc),
            //"ass_var_aoj_tr" => ass_var_aoj_tr(&mut sc),
            //"ass_var_aoj" => ass_var_aoj(&mut sc),
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
    docv = add_docxv(docv, DocxCompo::TableEnum(tb), vw)?;

    let pa = Paragraph::new().add_run(Run::new());
    docv = add_docxv(docv, DocxCompo::ParagEnum(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn v1(
    docv: Vec<DocxCompo>,
    //mut docv: Vec<DocxCompo>,
    _pws: &[String],
    _vw: &Rc<RefCell<XmlElem>>
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    /*
    println!("view 1");
    let _sc = script_param(pws);
    let boxes = get_elem_boxes(vw);
    for dis in boxes.iter() {
        for di in dis.iter() {
            let elm = get_elem_elem(di);
        }
    }
    for dis in vw.vdia.iter() {
        for di in dis.iter() {
            docv = ar_gen_doc(docv, &di.eid, vw)?;
        }
    }
    */
    Ok(docv)
}

//======================================================
pub fn v2(
    //mut docv: Vec<DocxCompo>,
    docv: Vec<DocxCompo>,
    _pws: &[String],
    _vw: &Rc<RefCell<XmlElem>>,
    _hd: bool,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    println!("view 1");
    /*
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
    */
    Ok(docv)
}

//======================================================
pub fn e2(
    docv: Vec<DocxCompo>,
    //mut docv: Vec<DocxCompo>,
    _pws: &[String],
    _vw: &Rc<RefCell<XmlElem>>,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    /*
    let _sc = script_param(pws);
    for dis in vw.edia.iter() {
        for di in dis.iter() {
            docv = elm_to_render(docv, di.rc.clone(), vw, true)?;
        }
    }
    */
    Ok(docv)
}

//======================================================
pub fn e1(
    //mut docv: Vec<DocxCompo>,
    docv: Vec<DocxCompo>,
    _pws: &[String],
    _vw: &Rc<RefCell<XmlElem>>,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    /*
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
    */
    Ok(docv)
}

use std::sync::{LazyLock, Mutex};
use bincode::{Decode, Encode};

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

//======================================================
pub fn sp(mut docv: Vec<DocxCompo>, vw: &Rc<RefCell<XmlElem>>) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
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
    docv = add_docxv(docv, DocxCompo::ParagEnum(pa), vw)?;
    Ok(docv)
}


//======================================================
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

//======================================================
pub fn doc0(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &Rc<RefCell<XmlElem>>,
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
    docv = add_docxv(docv, DocxCompo::ParagEnum(c1), vw)?;
    Ok(docv)
}

//======================================================
pub fn img1(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &Rc<RefCell<XmlElem>>,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    let lst = pws.last().unwrap();
    let vid = get_elem_id(vw);
    println!("img1 {}", vid);
    let dnm = crate::dcl::get_dirnm();
    let fimg = format!("{dnm}/eaviews/{}.png", vid);
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
                docv = add_docxv(docv, DocxCompo::ParagEnum(ppic), vw)?;
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
        docv = add_docxv(docv, DocxCompo::ParagEnum(pa), vw)?;
    } else {
        println!("IMG {fimg} not found");
    }
    Ok(docv)
}

//======================================================
pub fn img2(
    mut docv: Vec<DocxCompo>,
    pws: &Vec<String>,
    vw: &Rc<RefCell<XmlElem>>,
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
            docv = add_docxv(docv, DocxCompo::ParagEnum(ppic), vw)?;
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
    docv = add_docxv(docv, DocxCompo::ParagEnum(pa), vw)?;
    Ok(docv)
}

//======================================================
pub fn img3(
    mut docv: Vec<DocxCompo>,
    pws: &[String],
    vw: &Rc<RefCell<XmlElem>>,
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
            docv = add_docxv(docv, DocxCompo::ParagEnum(ppic), vw)?;
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
    docv = add_docxv(docv, DocxCompo::ParagEnum(pa), vw)?;
    Ok(docv)
}

//======================================================
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

//=======================================================
pub fn add_docxv(
    mut docv: Vec<DocxCompo>,
    docx: DocxCompo,
    _vw: &Rc<RefCell<XmlElem>>,
) -> Result<Vec<DocxCompo>, Box<dyn Error>> {
    docv.push(docx);
    Ok(docv)
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

pub fn get_archi_info() -> Result<ArchiInfo, Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let arif = archi_analyze(xml)?;
    Ok(arif)
}

pub fn gen_doc1(vnm: &str, fnm: &str) -> Result<(), Box<dyn Error>> {
    let mut arif = get_archi_info()?;
    /*
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    */
    get_assum_in_view(vnm, &mut arif)?;
    //arif.init_assum();
    let dnm = arif.ass.t(OUTDIR);
    set_dirnm(&dnm);
    println!("dnm: {dnm}");

    let mut docv = create_docx_def0();
    docv = ar_gen_doc(docv, &arif)?;

    /*
    println!("vid: {vid}");
    archi_extract(INDIR, "archi")?;
    let (ar_elem, _ar_fold) = archi_ana1()?;
    //println!("elm:{} fold:{}", ar_elem.child.len(), _ar_fold.child.len());
    let mut ar_elmh = HashMap::<String, Rc<RefCell<ArchElem>>>::new();
    let mut ar_prof = HashMap::<String, String>::new();
    for e in ar_elem.child.iter() {
        let ee = e.borrow();
        let Some(id) = ee.attr.get("id") else {
            continue;
        };
        if ee.elem == "profile" {
            let Some(nm) = ee.attr.get("name") else {
                continue;
            };
            //println!("profile = '{nm}'");
            ar_prof.insert(nm.to_string(), id.to_string());
        }
        ar_elmh.insert(id.to_string(), e.clone());
    }
    let mut docv = create_docx_def0();
    docv = ar_gen_doc(docv, vid, &ar_elmh)?;
    //println!("ARCHI5 : {} ======================", docv.len());
    */
    let fnm1 = format!("./temp/{fnm}0.docx");
    let fnm2 = format!("./temp/{fnm}.docx");
    write_docx(docv, fnm1.as_str(), fnm2.as_str(), "temp")?;
    Ok(())
}

pub fn attr_map(e: &quick_xml::events::BytesStart) -> HashMap<String, String> {
    let mut am = HashMap::<String, String>::new();
    for at in e.attributes().flatten() {
        let key = String::from_utf8(at.key.as_ref().to_vec()).unwrap_or_default();
        let val = match at.value {
            Cow::Borrowed(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
            Cow::Owned(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
        };
        am.insert(key, val);
    }
    am
}


/*
pub fn sg_proc2(coreno: usize, vnm: &str) -> Result<(), Box<dyn Error>> {
    println!("..sg_proc2 vnm:{vnm}");
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    let asrw = stage_02_1(coreno, &arif)?;
    stage_02_b(coreno, &arif, asrw)?;
    Ok(())
}
*/

pub fn sg_proc3(coreno: usize, vnm: &str) -> Result<(), Box<dyn Error>> {
    println!("..sg_proc2 vnm:{vnm}");
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    //init_view(vnm, &mut arif)?;
    let asrw = stage_02_1(coreno, &arif)?;
    //stage_02_b(coreno, &arif, asrw)?;
    let asrw = stage_02_c(coreno, &arif, asrw)?;
    stage_02_d(coreno, &arif, asrw)?;
    Ok(())
}

// smart grid process step 2 and save
pub fn sg_proc4(coreno: usize, vnm: &str) -> Result<(), Box<dyn Error>> {
    //println!("..sg_proc4 vnm:{vnm}");
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    //init_view(vnm, &mut arif)?;
    let asrw = stage_02_1(coreno, &arif)?;
    //stage_02_b(coreno, &arif, asrw)?;
    let tras_raw = stage_02_c(coreno, &arif, asrw)?;
    {
        let tik = std::time::SystemTime::now();
        let dnm = arif.ass.t(OUTDIR);
        let fnm = format!("{dnm}/all-rw4.bin");
        let bin: Vec<u8> = bincode::encode_to_vec(&tras_raw, bincode::config::standard()).unwrap();
        std::fs::write(fnm, bin).unwrap();
        let se = tik.elapsed().unwrap().as_secs();
        println!("SAVE ALL:{} - {se}sec", tras_raw.len());
    }
    Ok(())
}

// smart grid process step 3 and group
pub fn sg_proc5(coreno: usize, vnm: &str) -> Result<(), Box<dyn Error>> {
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
    stage_02_d(coreno, &arif, assv)?;
    Ok(())
}

pub fn sg_proc1(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    println!("..sg_proc1 vnm:{vnm}");
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    stage_01()?;
    Ok(())
}
    /*
    let _dias = get_diag_in_view(vnm, &arif)?;
    println!("mm {}", dias.len());
    for di0 in dias.iter() {
        let di = di0.borrow();
        if let XmlElem::Elem{tg: ArchiTag::child,em} = &*di {
            println!("check id:{} el:{} do:{}", em.ex.id, em.ex.elm_id, em.ex.mod_id);
            if let Some(ee) = arif.id_elem.get(&em.ex.elm_id) {
                println!(" ELEM");
                let eb = ee.borrow();
                if let XmlElem::Elem{..} = &*eb {
                    println!("d:{:?}", em.ex);
                }
            }
            if let Some(ee) = arif.id_elem.get(&em.ex.mod_id) {
                println!(" MODEL");
                let eb = ee.borrow();
                if let XmlElem::Elem{..} = &*eb {
                    println!("v:{:?}", em.ex);
                }
            }
        }
    }
    */

use crate::utl4::NumValEnum;
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct AssumObject {
    pub name: String,
    pub dlgid: String,
    pub elmid: String,
    pub trgs: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AssumValue {
    pub sid: String,
    pub tid: String,
    pub val: NumValEnum,
    pub name: String,
}

pub fn get_assum_in_view(vnm: &str, arif: &mut ArchiInfo) -> Result<(), Box<dyn Error>> {
    //let mut mem = Vec::<Rc<RefCell<XmlElem>>>::new();
    arif.view = vnm.to_string();
    let Some(vw) = arif.nm_view.get(vnm) else {
        return Err("ERROR utl6-#2".into());
    };
    arif.e_view = Some(vw.clone());
    let mut ass_m = HashMap::<String, NumValEnum>::new();
    let mut scns = vec![vw.clone()];
    //let (mut c1,mut c2) = (0,0);
    let mut ass_obj = Vec::<AssumObject>::new();
    let mut ass_val = Vec::<AssumValue>::new();
    while let Some(scin0) = scns.pop() {
        let scin = scin0.borrow();
        match &*scin {
            XmlElem::Elem{tg: ArchiTag::element, em:em1} |
            XmlElem::Elem{tg: ArchiTag::child, em:em1} => {
                match em1.ex.xsitp {
                    ArchiType::DiagramModelReference => {
                        //println!("obj2 #{:?} m:{} x:{:?}", em.ex.xsitp, em.ex.mod_id, em.ex);
                        if let Some(vw2) = arif.id_elem.get(&em1.ex.mod_id) {
                            //println!("  ===>");
                            scns.push(vw2.clone());
                        }
                    }
                    ArchiType::DiagramObject => {
                        if let Some(e) = arif.id_elem.get(&em1.ex.elm_id) {
                            let ee = e.borrow();
                            if let XmlElem::Elem{tg:_,em:em2} = &*ee && let Some(pfe) = arif.id_prof.get(&em2.ex.prf_id) {
                                let pfee = pfe.borrow();
                                if let XmlElem::Elem{tg:_,em: pfem} = &*pfee {
                                    let xr = format!("{:?}_{}", em2.ex.xsitp, pfem.ex.name);
                                    if let Ok(prf) = ArchiProfile::from_str(&xr) {
                                        match prf {
                                            ArchiProfile::BusinessObject_Assum => {
                                                let name = em2.ex.name.clone();
                                                let dlgid = em1.ex.id.clone();
                                                let elmid = em2.ex.id.clone();
                                                let trgs = em1.ex.trgs.clone();
                                                //println!("  1. xr: {xr} - {prf:?} - nm:{name} d:{dlgid} e:{elmid} t:{tagid}");
                                                let asob = AssumObject { name,dlgid,elmid,trgs };
                                                ass_obj.push(asob);
                                            }
                                            ArchiProfile::Artifact_AssumNum => {
                                                let name = em2.ex.name.to_string();
                                                let name = name.replace(",", "");
                                                let name = name.replace("_", "");
                                                let name = name.replace("f32", "");
                                                let val = if let Ok(vf32) = name.parse::<f32>() {
                                                    NumValEnum::Real(vf32)
                                                } else {
                                                    NumValEnum::None
                                                };
                                                for sc in em1.ex.src_cons.iter() {
                                                    let name = name.clone();
                                                    let val = val.clone();
                                                    let sid = sc.src_id.to_string();
                                                    let tid = sc.trg_id.to_string();
                                                    let asva = AssumValue { sid,tid, name, val, };
                                                    //println!("  2. xr: {xr} - {prf:?} - nm:{} s:{asva:?}", em2.ex.name);
                                                    ass_val.push(asva);
                                                }
                                                //println!(" AssumValue: {ssid}");
                                            }
                                            ArchiProfile::Artifact_AssumText => {
                                                //println!("  3. xr: {xr} - {prf:?} - nm:{}", em.ex.name);
                                                let name = em2.ex.name.to_string();
                                                let val = NumValEnum::Text(name.clone());
                                                for sc in em1.ex.src_cons.iter() {
                                                    let name = name.clone();
                                                    let val = val.clone();
                                                    let sid = sc.src_id.to_string();
                                                    let tid = sc.trg_id.to_string();
                                                    let asva = AssumValue { sid,tid, name, val, };
                                                    ass_val.push(asva);
                                                }
                                                //println!("  3. xr: {xr} - {prf:?} - nm:{nm} s:{asva:?}");
                                             }
                                            ArchiProfile::Artifact_AssumJson => {
                                                let name = em2.ex.doc.to_string();
                                                let name = name.replace("&quot;", "\"");
                                                /*
                                                let name = em2.ex.name.to_string();
                                                let mut name = name.replace("&quot;", "\"");
                                                if name == "value" {
                                                    name = em2.ex.doc.to_string();
                                                }
                                                */
                                                if let Ok(v) = serde_json::from_str::<Value>(name.as_str()) {
                                                    let val = NumValEnum::Json(v.clone());
                                                    for sc in em1.ex.src_cons.iter() {
                                                        let name = name.clone();
                                                        let val = val.clone();
                                                        let sid = sc.src_id.to_string();
                                                        let tid = sc.trg_id.to_string();
                                                        let asva = AssumValue { sid,tid, name, val, };
                                                        ass_val.push(asva);
                                                    }
                                                } else {
                                                    println!("  JSON ERROR. xr: {xr} - {prf:?} - nm:{vnm}");
                                                    println!("    name: {}", em2.ex.name);
                                                    println!("    json: {}", em2.ex.doc);
                                                }
                                            }
                                            /*
                                            ArchiProfile::Artifact_AssumToml => {
                                                //println!("  5. xr: {xr} - {prf:?} - nm:{}", em.ex.name);
                                                c2 += 1;
                                            }
                                            */
                                            _ => {}
                                        }
                                        //println!("  6. xr: {xr} - {prf:?} - nm:{}", em.ex.name);
                                    } else {
                                        println!("   profile error #2 {xr}");
                                    }
                                }
                            }
                        }
                        //mem.push(scin0.clone());
                        for ch in em1.chd.iter() {
                            scns.push(ch.clone());
                        }
                    }
                    _ => {
                        //println!("obj0 #{:?}", em.ex.xsitp);
                        for ch in em1.chd.iter() {
                            scns.push(ch.clone());
                        }
                    }
                }
           }
            _ => {}
        }
    }
    //println!("cn: o:{} v:{}", ass_obj.len(), ass_val.len());

    //let mut obnms = HashSet::<String>::new();
    let mut obtgs = HashMap::<String, AssumObject>::new();
    //println!("============ OBJECT ===========");
    for ob in ass_obj.iter() {
        obtgs.insert(ob.dlgid.clone(), ob.clone());
        //if let Some(ob) = obnms.get(&ob.name) {
        //    println!("================ ASSU OBJ DUPLICATE : {ob:?}");
        //}
        //obnms.insert(ob.name.clone());
    }
    //println!("==== obj targs {}", obtgs.len());
    
    //println!("============ VALUE ===========");
    for va in ass_val.iter() {
        //println!("===VALUE {va:?}");
        if let Some(ob) = obtgs.get(&va.tid) {
            //println!(" OBJECT {} - {:?}", ob.name, va.val);
            ass_m.insert(ob.name.clone(), va.val.clone());
        }
    }
    let cn = ASM::iter().map(|_| 1).sum();
    arif.ass.v_assum = vec![NumValEnum::None; cn];
    arif.ass.v_asm_a = vec![0usize; cn];
    arif.ass.v_asm_r = vec![0f32; cn];
    arif.ass.v_asm_i = vec![0i32; cn];
    arif.ass.v_asm_s = vec!["".to_string(); cn];
    arif.ass.v_asm_u = vec![0usize; cn];
    for (k,v) in ass_m.iter() {
        //println!("=====  k:{k} value:{v:?}");
        if let Ok(ke) = ASM::from_str(k) {
            let ii = ke as usize;
            arif.ass.v_assum[ii] = v.clone();
            match v {
                NumValEnum::Real(v) => {
                    arif.ass.v_asm_r[ii] = *v;
                    arif.ass.v_asm_a[ii] += 1;
                }
                NumValEnum::Int(v) => {
                    arif.ass.v_asm_i[ii] = *v;
                    arif.ass.v_asm_a[ii] += 1;
                }
                NumValEnum::Text(v) => {
                    arif.ass.v_asm_s[ii] = v.to_string();
                    arif.ass.v_asm_a[ii] += 1;
                }
                _ => {
                    //println!("NEED DATA CONTAINER FOR this type '{k}' -> {v:?}");
                }
            }
        } else {
            println!("#####=======  UNSUPPORT ASSUMTION {k}");
        }
    }
    arif.ass.nm_assu = ass_m;
    if arif.ass.v_asm_a[ASM::OUTDIR as usize]>0 {
        let dnm = arif.ass.t(crate::asm::ASM::OUTDIR);
        crate::dcl::set_dirnm(&dnm);
    }
    Ok(())
}

pub fn get_diag_in_view(vw: Rc<RefCell<XmlElem>>) -> Result<Vec<Rc<RefCell<XmlElem>>>, Box<dyn Error>> {
    let mut mem = Vec::<Rc<RefCell<XmlElem>>>::new();
    let mut scn = vec![vw.clone()];
    while let Some(scin0) = scn.pop() {
        let scin = scin0.borrow();
        match &*scin {
            XmlElem::Elem{tg: ArchiTag::element, em} |
            XmlElem::Elem{tg: ArchiTag::child, em} => {
                match em.ex.xsitp {
                    ArchiType::DiagramModelReference |
                    ArchiType::DiagramObject => {
                        mem.push(scin0.clone());
                    }
                    _ => {}
                }
                for ch in em.chd.iter() {
                    //println!("mem :{ee:?}");
                    scn.push(ch.clone());
                }
            }
            _ => {}
        }
    }
    Ok(mem)
}

pub fn archi_analyze(xml: Rc<RefCell<XmlElem>>) -> Result<ArchiInfo, Box<dyn Error>> {
    let mut archi = ArchiInfo::new("");
    archi.e_root = Some(xml.clone());
    let rex = regex::Regex::new(r".+:(.+)").unwrap();

    let mut xmls = vec![xml.clone()];
    while let Some(xml0) = xmls.pop() {
        let mut xml = xml0.borrow_mut();
        match &mut *xml {
            XmlElem::Elem{tg: ArchiTag::model,em} |
            XmlElem::Elem{tg: ArchiTag::folder,em}  => {
                if let Some(nm) = em.attr.get("name") {
                    em.ex.name = nm.to_string();
                }
                for c in em.chd.iter() {
                    xmls.push(c.clone());
                }
            }
            XmlElem::Elem{tg: ArchiTag::element,em} |
            XmlElem::Elem{tg: ArchiTag::child,em} => {
                if let Some(xsitp) = em.attr.get("xsi:type") {
                    em.ex.tp = xsitp.to_string();
                }
                if let Some(cap) = rex.captures_iter(&em.ex.tp).next() {
                    let tp = &cap[1].to_string();
                    //if let Some(xsitp) = XSI_TYPES.get(tp) {
                    if let Ok(xsitp) = ArchiType::from_str(tp.as_str()) {
                        em.ex.xsitp = xsitp.clone();
                    } else {
                        println!("  tp error2 {}", em.ex.tp);
                    }
                }
                if let Some(id) = em.attr.get("id") {
                    em.ex.id = id.to_string();
                    archi.id_elem.insert(id.to_string(), xml0.clone());
                }
                if let Some(nm) = em.attr.get("name") {
                    em.ex.name = nm.to_string();
                }
                if let ArchiType::ArchimateDiagramModel = em.ex.xsitp && !em.ex.name.is_empty() {
                    archi.nm_view.insert(em.ex.name.clone(), xml0.clone());
                }
                if let Some(elm_id) = em.attr.get("archimateElement") {
                    em.ex.elm_id = elm_id.to_string();
                }
                if let Some(img_pt) = em.attr.get("imagePath") {
                    em.ex.img_pt = img_pt.to_string();
                }
                if let Some(mod_id) = em.attr.get("model") {
                    em.ex.mod_id = mod_id.to_string();
                }
                if let Some(prf_id) = em.attr.get("profiles") {
                    em.ex.prf_id = prf_id.to_string();
                }
                if let Some(trgs) = em.attr.get("targetConnections") {
                    for t in trgs.split(",") {
                        em.ex.trgs.push(t.to_string());
                    }
                }
                for c in em.chd.iter() {
                    xmls.push(c.clone());
                }
            }
            XmlElem::Elem{tg: ArchiTag::profile,em} => {
                if let (Some(id),Some(tp)) = (em.attr.get("id"),em.attr.get("conceptType")) {
                    let nm = if let Some(nm) = em.attr.get("name") { nm.to_string() } else { "".to_string() };
                    em.ex.name = nm;
                    em.ex.id = id.to_string();
                    em.ex.tp = tp.to_string();
                    if let Ok(xsitp) = ArchiType::from_str(tp.as_str()) {
                        em.ex.xsitp = xsitp.clone();
                    } else {
                        println!("  tp error1 {}", em.ex.tp);
                    }
                    archi.id_prof.insert(em.ex.id.to_string(), xml0.clone());
                } else {
                    println!("   profile error");
                }
            }
            XmlElem::Elem{tg: ArchiTag::sourceConnection,em} => {
                if let Some(pa) = &em.pa {
                    let mut pa = pa.borrow_mut();
                    if let XmlElem::Elem{em:pem,..} = &mut *pa {
                        let id = if let Some(id) = em.attr.get("id") { id.to_string() } else { "".to_string() };
                        let xsitp = if let Some(xsitp) = em.attr.get("xsi:type") { xsitp.to_string()} else { "".to_string() };
                        let xsitp = ArchiType::from_str(&xsitp).unwrap_or_default();
                        let rel_id = if let Some(rel_id) = em.attr.get("archimateRelationship") { rel_id.to_string()} else { "".to_string() };
                        let src_id = if let Some(src_id) = em.attr.get("source"){ src_id.to_string()} else { "".to_string() };
                        let trg_id = if let Some(trg_id) = em.attr.get("target") { trg_id.to_string()} else { "".to_string() };
                        let srcon = SourceConnection { id, xsitp, src_id, rel_id, trg_id };
                        pem.ex.src_cons.push(srcon);
                    }
                }

            }
            XmlElem::Elem{tg: ArchiTag::bounds,em} => {
                if let Some(pa) = &em.pa {
                    let mut pa = pa.borrow_mut();
                    if let XmlElem::Elem{em:pem,..} = &mut *pa {
                        if let Some(x) = em.attr.get("x") {
                            pem.ex.bnd.x = x.parse().unwrap();
                        }
                        if let Some(y) = em.attr.get("y") {
                            pem.ex.bnd.y = y.parse().unwrap();
                        }
                        if let Some(height) = em.attr.get("height") {
                            pem.ex.bnd.height = height.parse().unwrap();
                        }
                        if let Some(width) = em.attr.get("width") {
                            pem.ex.bnd.width = width.parse().unwrap();
                        }
                    }
                }
            }
            XmlElem::Elem{tg: ArchiTag::property,em} => {
                if let Some(k) = em.attr.get("key") {
                    let v = if let Some(v) = em.attr.get("value") { v.to_string() } else { "".to_string() };
                    if let Some(pa) = &em.pa {
                        let mut pa = pa.borrow_mut();
                        if let XmlElem::Elem{em,..} = &mut *pa {
                            em.ex.prop.push((k.to_string(),v.to_string()));
                        }
                    }
                } else {
                    println!("   property error {:?}",em.attr);
                }
            }
            XmlElem::Elem{tg: ArchiTag::content,em} |
            XmlElem::Elem{tg: ArchiTag::hintContent,em} |
            XmlElem::Elem{tg: ArchiTag::documentation,em} => {
                let mut txs = String::new();
                for d in em.chd.iter() {
                    use std::fmt::Write;
                    if let XmlElem::TextElem{tx} = &*d.borrow() {
                        //print!("-{tx}|");
                        write!(txs,"{tx}")?;
                    }
                    if let XmlElem::GenRef{tx} = &*d.borrow() {
                        let al = match tx.as_str() {
                            //"#xD" => "\n".to_string(),
                            "#xD" => "".to_string(),
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
                        write!(txs,"{al}")?;
                    }
                }
                if let Some(pa) = &em.pa {
                    let mut pa = pa.borrow_mut();
                    if let XmlElem::Elem{em,..} = &mut *pa {
                        em.ex.doc = txs.to_string();
                    }
                }
            }
            _ => { }
        }
    }
    //println!("e:{} v:{} p:{}", archi.id_elem.len(), archi.nm_view.len(), archi.id_prof.len());
    Ok(archi)
}

/*
pub fn archi_extract() -> Result<(), Box<dyn Error>> {
    let arfn = ARCHI_INPUT;
    let farch = fs::File::open(arfn).unwrap();
    let mut arch = zip::ZipArchive::new(farch).unwrap();

    fs::create_dir_all(ARCH_OUTDIR).unwrap();
    for i in 0..arch.len() {
        let mut file = arch.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        let outfn = outpath.display().to_string();
        //println!("outpath: {}", outfn);
        if outfn.starts_with("images/") {
            let dir = format!("{}/images", ARCH_OUTDIR);
            fs::create_dir_all(&dir).unwrap();
            //println!("dir: {dir}");
        }
        {
            let fnm = format!("{}/{}", ARCH_OUTDIR, outpath.display());
            //println!("fnm: {fnm}");
            let mut outfile = fs::File::create(&fnm).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }
    Ok(())
}
*/

pub fn archi_extract0(arfn: &str) -> Result<(), Box<dyn Error>> {
    let farch = fs::File::open(arfn).unwrap();
    let mut arch = zip::ZipArchive::new(farch).unwrap();

    fs::create_dir_all(ARCH_OUTDIR).unwrap();
    for i in 0..arch.len() {
        let mut file = arch.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        let outfn = outpath.display().to_string();
        //println!("outpath: {}", outfn);
        if outfn.starts_with("images/") {
            let dir = format!("{}/images", ARCH_OUTDIR);
            fs::create_dir_all(&dir).unwrap();
            //println!("dir: {dir}");
        }
        {
            let fnm = format!("{}/{}", ARCH_OUTDIR, outpath.display());
            //println!("fnm: {fnm}");
            let mut outfile = fs::File::create(&fnm).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }
    Ok(())
}

pub fn archi_xml_read0() -> Result<Rc<RefCell<XmlElem>>, Box<dyn Error>> {
    let fmod = format!("{ARCH_OUTDIR}/model.xml");
    let mut tagmap = HashMap::<Vec<u8>, ArchiTag>::new();
    for tg in ArchiTag::iter() {
        let tag = format!("{tg:?}").into_bytes();
        tagmap.insert(tag, tg);
    }
    let mut res = None;
    //let mut xmls = Vec::<Rc<RefCell<XmlElem>>>::new();
    if let Ok(mut xrd) = XmlReader::from_file(&fmod) {
        let mut xbuf = Vec::new();
        let mut stck = Vec::<Rc<RefCell<XmlElem>>>::new();
        loop {
            match xrd.read_event_into(&mut xbuf) {
                Ok(Event::Eof) => break,
                Ok(Event::Empty(e)) => {
                    if let Some(tg) = tagmap.get(e.local_name().as_ref()) {
                        let tg = tg.clone();
                        let mut em = ElemInfo {
                            attr: attr_map(&e),
                            ..Default::default()
                        };
                        if !stck.is_empty() {
                            let pa = stck[stck.len() - 1].clone();
                            em.pa = Some(pa);
                        }
                        let xml = XmlElem::Elem { tg, em };
                        let xml2 = Rc::new(RefCell::new(xml));
                        //let xml4 = xml2.clone();
                        if !stck.is_empty() {
                            let pa = stck[stck.len() - 1].clone();
                            let mut xml3 = pa.borrow_mut();
                            if let XmlElem::Elem { em, .. } = &mut *xml3 {
                                em.chd.push(xml2);
                            }
                        }
                        //xmls.push(xml4);
                    } else {
                        println!("Start not found {e:?}");
                    }
                }
                Ok(Event::Start(e)) => {
                    if let Some(tg) = tagmap.get(e.local_name().as_ref()) {
                        let tg = tg.clone();
                        let mut em = ElemInfo {
                            attr: attr_map(&e),
                            ..Default::default()
                        };
                        if !stck.is_empty() {
                            let pa = stck[stck.len() - 1].clone();
                            em.pa = Some(pa);
                        }
                        let xml = XmlElem::Elem { tg, em };
                        let xml2 = Rc::new(RefCell::new(xml));
                        let xml4 = xml2.clone();
                        if !stck.is_empty() {
                            let pa = stck[stck.len() - 1].clone();
                            let mut xml3 = pa.borrow_mut();
                            if let XmlElem::Elem { em, .. } = &mut *xml3 {
                                em.chd.push(xml2);
                            }
                        }
                        stck.push(xml4.clone());
                        if res.is_none() {
                            res = Some(xml4);
                        }
                        //xmls.push(xml4);
                    } else {
                        println!("Start not found {e:?}");
                    }
                }
                Ok(Event::End(_)) => {
                    stck.pop();
                }
                Ok(Event::Text(tx)) => {
                    let tx = String::from_utf8(tx.to_vec()).unwrap_or_default();
                    let xml = XmlElem::TextElem { tx };
                    let xml = Rc::new(RefCell::new(xml));
                    if !stck.is_empty() {
                        let pa = stck[stck.len() - 1].clone();
                        let mut xml3 = pa.borrow_mut();
                        if let XmlElem::Elem { em, .. } = &mut *xml3 {
                            em.chd.push(xml);
                        }
                    }
                }
                Ok(Event::GeneralRef(ge)) => {
                    let tx = String::from_utf8(ge.to_vec()).unwrap_or_default();
                    let xml = XmlElem::GenRef { tx };
                    let xml = Rc::new(RefCell::new(xml));
                    if !stck.is_empty() {
                        let pa = stck[stck.len() - 1].clone();
                        let mut xml3 = pa.borrow_mut();
                        if let XmlElem::Elem { em, .. } = &mut *xml3 {
                            em.chd.push(xml);
                        }
                    }
                }
                Ok(Event::Decl(_de)) => {
                    /*
                    let tx = String::from_utf8(de.to_vec()).unwrap_or_default();
                    let xml = XmlElem::XmlDecl(tx);
                    xmls.push(Rc::new(RefCell::new(xml)));
                    */
                }
                e => {
                    println!("========= ERROR 6 : {:?}", e);
                }
            }
        }
    }
    if let Some(x) = res {
        Ok(x)
    } else {
        Err("NO XML DATA".into())
    }
}

pub fn archi_xml_read() -> Result<Vec<Rc<RefCell<XmlElem>>>, Box<dyn Error>> {
    let fmod = format!("{ARCH_OUTDIR}/model.xml");
    let mut tagmap = HashMap::<Vec<u8>, ArchiTag>::new();
    for tg in ArchiTag::iter() {
        let tag = format!("{tg:?}").into_bytes();
        tagmap.insert(tag, tg);
    }
    let mut xmls = Vec::<Rc<RefCell<XmlElem>>>::new();
    if let Ok(mut xrd) = XmlReader::from_file(&fmod) {
        let mut xbuf = Vec::new();
        let mut stck = Vec::<Rc<RefCell<XmlElem>>>::new();
        loop {
            match xrd.read_event_into(&mut xbuf) {
                Ok(Event::Eof) => break,
                Ok(Event::Empty(e)) => {
                    if let Some(tg) = tagmap.get(e.local_name().as_ref()) {
                        let tg = tg.clone();
                        let mut em = ElemInfo {
                            attr: attr_map(&e),
                            ..Default::default()
                        };
                        if !stck.is_empty() {
                            let pa = stck[stck.len() - 1].clone();
                            em.pa = Some(pa);
                        }
                        let xml = XmlElem::Elem { tg, em };
                        let xml2 = Rc::new(RefCell::new(xml));
                        let xml4 = xml2.clone();
                        if !stck.is_empty() {
                            let pa = stck[stck.len() - 1].clone();
                            let mut xml3 = pa.borrow_mut();
                            if let XmlElem::Elem { em, .. } = &mut *xml3 {
                                em.chd.push(xml2);
                            }
                        }
                        xmls.push(xml4);
                    } else {
                        println!("Start not found {e:?}");
                    }
                }
                Ok(Event::Start(e)) => {
                    if let Some(tg) = tagmap.get(e.local_name().as_ref()) {
                        let tg = tg.clone();
                        let mut em = ElemInfo {
                            attr: attr_map(&e),
                            ..Default::default()
                        };
                        if !stck.is_empty() {
                            let pa = stck[stck.len() - 1].clone();
                            em.pa = Some(pa);
                        }
                        let xml = XmlElem::Elem { tg, em };
                        let xml2 = Rc::new(RefCell::new(xml));
                        let xml4 = xml2.clone();
                        if !stck.is_empty() {
                            let pa = stck[stck.len() - 1].clone();
                            let mut xml3 = pa.borrow_mut();
                            if let XmlElem::Elem { em, .. } = &mut *xml3 {
                                em.chd.push(xml2);
                            }
                        }
                        stck.push(xml4.clone());
                        xmls.push(xml4);
                    } else {
                        println!("Start not found {e:?}");
                    }
                }
                Ok(Event::End(_)) => {
                    stck.pop();
                }
                Ok(Event::Text(tx)) => {
                    let tx = String::from_utf8(tx.to_vec()).unwrap_or_default();
                    let xml = XmlElem::TextElem { tx };
                    let xml = Rc::new(RefCell::new(xml));
                    if !stck.is_empty() {
                        let pa = stck[stck.len() - 1].clone();
                        let mut xml3 = pa.borrow_mut();
                        if let XmlElem::Elem { em, .. } = &mut *xml3 {
                            em.chd.push(xml);
                        }
                    }
                }
                Ok(Event::GeneralRef(ge)) => {
                    let tx = String::from_utf8(ge.to_vec()).unwrap_or_default();
                    let xml = XmlElem::GenRef { tx };
                    let xml = Rc::new(RefCell::new(xml));
                    if !stck.is_empty() {
                        let pa = stck[stck.len() - 1].clone();
                        let mut xml3 = pa.borrow_mut();
                        if let XmlElem::Elem { em, .. } = &mut *xml3 {
                            em.chd.push(xml);
                        }
                    }
                }
                Ok(Event::Decl(_de)) => {
                    /*
                    let tx = String::from_utf8(de.to_vec()).unwrap_or_default();
                    let xml = XmlElem::XmlDecl(tx);
                    xmls.push(Rc::new(RefCell::new(xml)));
                    */
                }
                e => {
                    println!("========= ERROR 3 : {:?}", e);
                }
            }
        }
    }
    Ok(xmls)
}

#[derive(Debug, Clone, Default)]
pub struct ArchiXmlInfo {
    pub image_paths: HashSet<String>,
}

pub fn archi_xml_write(
    lv: usize,
    xx: &mut String,
    el: Rc<RefCell<XmlElem>>,
    ai: &mut ArchiXmlInfo,
) -> Result<(), Box<dyn Error>> {
    let el = el.borrow();
    if let XmlElem::Elem { em, tg } = &*el {
        use std::fmt::Write;
        let _ = (0..lv).map(|_| write!(xx, "  "));
        let mut tg = format!("{tg:?}");
        if tg == "model" {
            tg = "archimate:model".to_string();
        }
        write!(xx, "<{tg}")?;
        for (k, v) in &em.attr {
            write!(xx, " {k}=\"{v}\"")?;
            if k == "imagePath" {
                ai.image_paths.insert(v.to_string());
            }
        }
        write!(xx, ">")?;
        for em in &em.chd {
            let em2 = em.clone();
            let ee = em.borrow();
            match &*ee {
                XmlElem::TextElem { tx } => {
                    //if !tx.trim().is_empty() {
                    write!(xx, "{tx}")?;
                    //}
                }
                XmlElem::GenRef { tx } => {
                    write!(xx, "&{tx};")?;
                }
                XmlElem::Elem { .. } => {
                    let mut done = false;
                    let em0 = em2.clone();
                    let em0 = em0.borrow();
                    if let XmlElem::Elem { em, tg } = &*em0 && em.chd.is_empty() {
                        write!(xx, "<{tg:?}")?;
                        for (k, v) in &em.attr {
                            write!(xx, " {k}=\"{v}\"")?;
                            if k == "imagePath" {
                                ai.image_paths.insert(v.to_string());
                            }
                        }
                        write!(xx, "/>")?;
                        done = true;
                    }
                    if !done {
                        archi_xml_write(lv + 1, xx, em2.clone(), ai)?;
                    }
                }
                _ => {}
            }
        }
        write!(xx, "</{tg}>")?;
    }
    Ok(())
}

pub fn archi_out0(xml: Rc<RefCell<XmlElem>>, fout: &str) -> Result<(), Box<dyn Error>> {
    let mut tx = String::new();
    use std::fmt::Write;
    writeln!(tx, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    fs::create_dir_all(format!("{ARCH_GENDIR}/images")).unwrap();
    let mut arc = ArchiXmlInfo::default();
    archi_xml_write(0, &mut tx, xml, &mut arc)?;

    let fout = std::fs::File::create(fout).unwrap();
    //let fout = std::fs::File::create("out.archimate").unwrap();
    let mut zout = zip::ZipWriter::new(fout);
    zout.add_directory("images", SimpleFileOptions::default())?;
    zout.start_file("model.xml", SimpleFileOptions::default())?;
    let modbuf = tx.into_bytes();
    zout.write_all(&modbuf)?;
    for i in arc.image_paths.iter() {
        let fi = format!("{ARCH_OUTDIR}/{i}");
        let metadata = std::fs::metadata(&fi)?;
        let mut buff = vec![0; metadata.len() as usize];
        let mut f = std::fs::File::open(&fi)?;
        f.read_exact(&mut buff)?;
        zout.start_file(i, SimpleFileOptions::default())?;
        zout.write_all(&buff).expect("?");
    }
    zout.finish().expect("?");

    Ok(())
}

pub fn archi_out(xmls: &[Rc<RefCell<XmlElem>>], fout: &str) -> Result<(), Box<dyn Error>> {
    let mut tx = String::new();
    use std::fmt::Write;
    writeln!(tx, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    let fst = xmls[0].clone();
    fs::create_dir_all(format!("{ARCH_GENDIR}/images")).unwrap();
    let mut arc = ArchiXmlInfo::default();
    archi_xml_write(0, &mut tx, fst, &mut arc)?;

    let fout = std::fs::File::create(fout).unwrap();
    //let fout = std::fs::File::create("out.archimate").unwrap();
    let mut zout = zip::ZipWriter::new(fout);
    zout.add_directory("images", SimpleFileOptions::default())?;
    zout.start_file("model.xml", SimpleFileOptions::default())?;
    let modbuf = tx.into_bytes();
    zout.write_all(&modbuf)?;
    for i in arc.image_paths.iter() {
        let fi = format!("{ARCH_OUTDIR}/{i}");
        let metadata = std::fs::metadata(&fi)?;
        let mut buff = vec![0; metadata.len() as usize];
        let mut f = std::fs::File::open(&fi)?;
        f.read_exact(&mut buff)?;
        zout.start_file(i, SimpleFileOptions::default())?;
        zout.write_all(&buff).expect("?");
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

    docv.push(DocxCompo::StyleEnum(Box::new(style1)));
    docv.push(DocxCompo::StyleEnum(Box::new(style2)));
    docv.push(DocxCompo::StyleEnum(Box::new(style3)));
    docv.push(DocxCompo::HeaderEnum(header));
    docv.push(DocxCompo::FooterEnum(footer));
    docv
}

pub async fn page(vnm: &str) -> Result<(), Box<dyn Error>> {
    archi_extract0(ARCHI_INPUT)?;
    let xml = archi_xml_read0()?;
    let mut arif = archi_analyze(xml)?;
    get_assum_in_view(vnm, &mut arif)?;
    //init_view(vnm, &mut arif)?;

    let dnm = arif.ass.t(crate::asm::ASM::OUTDIR);
    println!("dnm:{dnm}");
    crate::dcl::set_dirnm(&dnm);
    println!("page");
    //let x: axum::routing::MethodRouter = get(crate::web::sbb01::sbb01);
    let app = axum::Router::new()
        .route("/fdw01x", get(crate::web::fdw01x::page))
        .route("/fda01x", get(crate::img::fda01x::get_image_x))
        // sub image
        .route("/fda01", get(crate::img::fda01::get_image))
        .route("/fda02", get(crate::img::fda02::get_image))
        .route("/fda03", get(crate::img::fda03::get_image))
        .route("/fdw01", get(crate::web::fdw01::page))
        .route("/fdw02", get(crate::web::fdw02::page))
        .route("/fdw03", get(crate::web::fdw03::page))
        // sub feeder
        .route("/tra01", get(crate::web::tra01::page))
        // field
        .route("/sbb01", get(crate::web::sbb01::page))
        .route("/sbb02", get(crate::web::sbb02::page))
        .route("/sbb03", get(crate::web::sbb03::page))
        .route("/sbb04", get(crate::web::sbb04::page))
        .route("/sbb05", get(crate::web::sbb05::page))
        .route("/sbb06", get(crate::web::sbb06::page))
        .route("/sbb07", get(crate::web::sbb07::page))
        .route("/sbb08", get(crate::web::sbb08::page))
        .route("/sbb09", get(crate::web::sbb09::page))
        .route("/sbb10", get(crate::web::sbb10::page))
        .route("/sbb11", get(crate::web::sbb11::page))
        .route("/sbb12", get(crate::web::sbb12::page))
        .route("/sbb13", get(crate::web::sbb13::page))
        .route("/sbb14", get(crate::web::sbb14::page))
        .route("/sbb15", get(crate::web::sbb15::page))
        .route("/sbb16", get(crate::web::sbb16::page))
        // sub
        .route("/sba01", get(crate::sba01::sba01))
        .route("/sba02", get(crate::sba02::sba02))
        .route("/sba03", get(crate::sba03::sba03))
        // sub
        .route("/sb01", get(crate::sb01::sb01))
        .route("/sb02", get(crate::sb02::sb02))
        .route("/sb03", get(crate::sb03::sb03))
        .route("/sb04", get(crate::sb04::sb04))
        .route("/sb05", get(crate::sb05::sb05))
        // trans
        .route("/tr01", get(crate::tr01::tr01))
        .route("/tr02", get(crate::tr02::tr02))
        .route("/tr03", get(crate::tr03::tr03))
        .route("/tr04", get(crate::tr04::tr04))
        .route("/tr05", get(crate::tr05::tr05))
        .route("/tr06", get(crate::tr06::tr06))
        // ___
        .route("/", get(crate::sba01::sba01));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

pub fn excel_repo1(arif: &ArchiInfo, tp: i32) -> Result<(), Box<dyn Error>> {
    //println!("prv {mxrw:?}");
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
    let fnm = match tp {1=> "pvrw", 2=> "sbrw", 3=> "pvbrn", 4=> "branch", 5=> "pvbrn1", 6=> "branch1", 7=> "pvbrn2", 8=> "branch2", _=> "____"};
    let hed = match tp {1|3|5|7=> "จังหวัด", 2=> "สถานีไฟฟ้า", 4|6|8=> "กฟส",_=> "____" };

    let mut workbook = Workbook::new();
    let flds = sheets.clone();
    let Ok(buf) = std::fs::read(format!("{dnm}/000-{fnm}.bin")) else {
        return Err(format!("Not found {fnm}").into());
    };
    println!("===============  READ DATA FILE {fnm}");
    // ==== read rw3 data
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Not decoded".into());
    };

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

    let (brnv,brni) = get_brn_map()?;
    match tp {
        1|3|5|7 => {
            if !prvs.is_empty() {
                let mut pv_ix = HashMap::<String,usize>::new();
                for (i,ass) in assv0.iter().enumerate() {
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
                assv0.sort_by(|a,b| a.pvid.cmp(&b.pvid));
            }
        }
        2 => {
            assv0.sort_by(|a,b| a.pvid.cmp(&b.pvid));
        }
        4|6|8 => {
            if !brnv.is_empty() {
                let mut brn_ix = HashMap::<String,usize>::new();
                for (i,ass) in assv0.iter().enumerate() {
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
        _ => {}
    }
    /*
    if prvs.is_empty() {
        assv0.sort_by(|a,b| {
            let a0 = a.v[VarType::FirCstRate.tousz()].v;
            let b0 = b.v[VarType::FirCstRate.tousz()].v;
            b0.partial_cmp(&a0).unwrap()
        })
    } else {
        let mut pv_ix = HashMap::<String,usize>::new();
        for (i,ass) in assv0.iter().enumerate() {
            pv_ix.insert(ass.pvid.to_string(), i);
        }
        let mut assv1 = Vec::<PeaAssVar>::new();
        for pv in prvs.iter() {
            if let Some(iu) = pv_ix.get(pv) {
                //let x = assv0[*iu].clone();
                assv1.push(assv0[*iu].clone());
            }
        }
        assv0 = assv1;
    }
    */
    if maxrw>0 {
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
        let vas = format!("{co:?}");
        let _ = sht.write_with_format(rw, i as u16 + 2, vas, &hdfm);
    }
    let mut fmts = vec![];
    for dt in datas.iter() {
        let dtn = format!("{dt:?}");
        if dtn.ends_with("MWh") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else {
            fmts.push(Format::new().set_num_format("#,##0"));
        }
    }
    use crate::dcl::Geo;
    for (i, ass) in assv0.iter().enumerate() {
        rw += 1;
        let mut co = 0;
        let no = i as i32 + 1;
        let _ = sht.write(rw, co, no);
        co += 1;
        let nm = match tp {
            1|3|5|7 => ass.pvid.clone(), 
            2 => {
                let map = if let Some(sbinf) = sbif.get(&ass.sbid) { sbinf.name.clone() } else { "".to_string() };
                let (x,y) = ass.n1d.n1d_2_latlon();
                format!("'จ.{}'-'{}'-'ส.{}' L:[{},{}]",ass.pvid,ass.sbid,map,x,y)
            }, 
            4|6|8 => {
                let (nm,iv,pv,sz) = if let Some(i) = brni.get(&ass.aojcd) {
                    let name = if brnv[*i].is_prv { format!("กฟจ.{}",brnv[*i].name) } else { format!("กฟส.{}", brnv[*i].name) };
                    let stock = if let Some(jv) = brnv[*i].i_stock { brnv[jv].name.clone() } else { "".to_string() };
                    let prov = if let Some(jv) = brnv[*i].i_prov { brnv[jv].name.clone() } else { "".to_string() };
                    (name, stock, prov, brnv[*i].size.to_string())
                } else { ("".to_string(), "".to_string(), "".to_string(), "".to_string()) };
                format!("'{}'-'{nm}' คลัง:{iv} จ.{pv} size:'{sz}'", ass.aojcd)
            }
            _=>"".to_string()
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
        if dtn.ends_with("MWh") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else {
            fmts.push(Format::new().set_num_format("#,##0"));
        }
    }
    for (i, ass) in assv0.iter().enumerate() {
        rw += 1;
        let mut co = 0;
        let no = i as i32 + 1;
        let _ = sht.write(rw, co, no);
        co += 1;
        let nm = match tp {
            1|3|5|7 => ass.pvid.clone(), 
            2 => {
                let map = if let Some(sbinf) = sbif.get(&ass.sbid) { sbinf.name.clone() } else { "".to_string() };
                let (x,y) = ass.n1d.n1d_2_latlon();
                format!("'จ.{}'-'{}'-'ส.{}' L:[{},{}]",ass.pvid,ass.sbid,map,x,y)
            }, 
            4|6|8 => {
                let (nm,iv,pv,sz) = if let Some(i) = brni.get(&ass.aojcd) {
                    let name = if brnv[*i].is_prv { format!("กฟจ.{}",brnv[*i].name) } else { format!("กฟส.{}", brnv[*i].name) };
                    let stock = if let Some(jv) = brnv[*i].i_stock { brnv[jv].name.clone() } else { "".to_string() };
                    let prov = if let Some(jv) = brnv[*i].i_prov { brnv[jv].name.clone() } else { "".to_string() };
                    (name, stock, prov, brnv[*i].size.to_string())
                } else { ("".to_string(), "".to_string(), "".to_string(), "".to_string()) };
                format!("'{}'-'{nm}' คลัง:{iv} จ.{pv} size:'{sz}'", ass.aojcd)
            }
            _=>"".to_string()
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

    let mut datas = vec![VarType::FirCstRate, VarType::CstCapEx, VarType::CstOpEx, VarType::CstCapOpEx,];
    for vt in CAPEX_FLDS.iter() { datas.push(vt.clone()); }
    for vt in OPEX_FLDS.iter() { datas.push(vt.clone()); }
    for (i, co) in datas.iter().enumerate() {
        sht.set_column_width(i as u16 + 2, 15)?;
        let vas = format!("{co:?}");
        let _ = sht.write_with_format(rw, i as u16 + 2, vas, &hdfm);
    }
    let mut fmts = vec![];
    //for dt in DATA_FLDS.iter() {
    for dt in datas.iter() {
        let dtn = format!("{dt:?}");
        if dtn.ends_with("MWh") {
            fmts.push(Format::new().set_num_format("#,##0.00"));
        } else {
            fmts.push(Format::new().set_num_format("#,##0"));
        }
    }
    for (i, ass) in assv0.iter().enumerate() {
        rw += 1;
        let mut co = 0;
        let no = i as i32 + 1;
        let _ = sht.write(rw, co, no);
        co += 1;
        let nm = match tp {
            1|3|5|7 => ass.pvid.clone(), 
            2 => {
                let map = if let Some(sbinf) = sbif.get(&ass.sbid) { sbinf.name.clone() } else { "".to_string() };
                let (x,y) = ass.n1d.n1d_2_latlon();
                format!("'จ.{}'-'{}'-'ส.{}' L:[{},{}]",ass.pvid,ass.sbid,map,x,y)
            }, 
            4|6|8 => {
                let (nm,iv,pv,sz) = if let Some(i) = brni.get(&ass.aojcd) {
                    let name = if brnv[*i].is_prv { format!("กฟจ.{}",brnv[*i].name) } else { format!("กฟส.{}", brnv[*i].name) };
                    let stock = if let Some(jv) = brnv[*i].i_stock { brnv[jv].name.clone() } else { "".to_string() };
                    let prov = if let Some(jv) = brnv[*i].i_prov { brnv[jv].name.clone() } else { "".to_string() };
                    (name, stock, prov, brnv[*i].size.to_string())
                } else { ("".to_string(), "".to_string(), "".to_string(), "".to_string()) };
                format!("'{}'-'{nm}' คลัง:{iv} จ.{pv} size:'{sz}'", ass.aojcd)
            }
            _=>"".to_string()
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
            let nm = match tp {
            1|3|5|7 => ass.pvid.clone(), 
            2 => {
                let map = if let Some(sbinf) = sbif.get(&ass.sbid) { sbinf.name.clone() } else { "".to_string() };
                let (x,y) = ass.n1d.n1d_2_latlon();
                format!("'จ.{}'-'{}'-'ส.{}' L:[{},{}]",ass.pvid,ass.sbid,map,x,y)
            }, 
            4|6|8 => {
                let (nm,iv,pv,sz) = if let Some(i) = brni.get(&ass.aojcd) {
                    let name = if brnv[*i].is_prv { format!("กฟจ.{}",brnv[*i].name) } else { format!("กฟส.{}", brnv[*i].name) };
                    let stock = if let Some(jv) = brnv[*i].i_stock { brnv[jv].name.clone() } else { "".to_string() };
                    let prov = if let Some(jv) = brnv[*i].i_prov { brnv[jv].name.clone() } else { "".to_string() };
                    (name, stock, prov, brnv[*i].size.to_string())
                } else { ("".to_string(), "".to_string(), "".to_string(), "".to_string()) };
                format!("'{}'-'{nm}' คลัง:{iv} จ.{pv} size:'{sz}'", ass.aojcd)
            }
                _=>"".to_string()
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


/*
//use std::fs::File;
//use std::path::Path;
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
    if let Ok(l1) = read_lines(&f1) && let Ok(l2) = read_lines(&f2) {
        for line in l1.map_while(Result::ok) {
*/

