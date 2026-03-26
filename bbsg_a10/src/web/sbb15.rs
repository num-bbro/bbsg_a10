use crate::dcl::Pan;
use crate::dcl::PeaAssVar;
use crate::dcl::VarType;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::Query;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Param {
    pub fld: Option<String>,
}

use crate::dcl::SHOW_FLDS3;
use crate::p08::ld_sub_info;
use crate::p08::SubInfo;
use std::collections::HashMap;

#[derive(Template, WebTemplate, Debug, Default)]
#[template(path = "sbb0t.html")]
pub struct WebTemp {
    name: String,
    assv: Vec<PeaAssVar>,
    sbif: HashMap<String, SubInfo>,
    shwfld: Vec<VarType>,
}

pub async fn page(_para: Query<Param>) -> WebTemp {
    let dnm = crate::dcl::get_dirnm();
    let name = "PROVINCE".to_string();
    let Ok(buf) = std::fs::read(format!("{dnm}/000-pvrw.bin")) else {
        println!("NO rw3.bin file:");
        return WebTemp::default();
    };
    //
    // ==== read rw3 data
    let Ok((mut assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        println!("Failed to decode rw3:");
        return WebTemp::default();
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
    let mut sumv = PeaAssVar::from(0u64);
    let mut assv = Vec::<PeaAssVar>::new();
    //for ass in assv0.iter().take(25) {
    for ass in assv0.iter() {
        sumv.add(ass);
        assv.push(ass.clone());
    }
    assv.push(sumv);
    let sbif = ld_sub_info();
    WebTemp {
        name,
        assv,
        sbif: sbif.clone(),
        shwfld: SHOW_FLDS3.to_vec(),
    }
}
