use crate::dcl::*;
use crate::utl::p01_chk;
use crate::utl::*;
use crate::wrt::write_trn_ass_02;
//use num::pow::Pow;
use crate::stx2::ass_calc;
use std::collections::{HashMap, HashSet};
use std::error::Error;
//use crate::cst1::cst_reinvest;
use crate::p08::ld_sub_calc;
//use axum::http::StatusCode;
use bincode::Decode;
use bincode::Encode;
use sglib04::geo3::GisAoj;
use sglib04::web1::OP_YEAR_END;
use sglib04::web1::OP_YEAR_START;

//pub const CALL_CENTER_COST_UP: f32 = 0.04f32;
pub const ASSET_WORTH_RATIO: f32 = 0.2f32;
pub const MODEL_ENTRY_RATIO: f32 = 0.05f32;
pub const MODEL_ENTRY_COST: f32 = 1000f32;
pub const RENEW_HOUR_PER_DAY: f32 = 4.0;
pub const RENEW_SAVE_PER_MWH: f32 = 500f32;
pub const PEAK_POWER_RATIO: f32 = 0.3;

pub const UNBAL_LOSS_CLAIM_RATE: f32 = 0.8;
pub const TRANS_REPL_CLAIM_RATE: f32 = 0.8;
pub const UNBAL_REPL_CLAIM_RATE: f32 = 0.8;
pub const NOTEC_LOSS_CLAIM_RATE: f32 = 0.8;

pub const NON_TECH_LOSS_RATIO: f32 = 0.02;
//pub const UNBAL_HOUR_PER_DAY: f32 = 4.0;
pub const UNBAL_HOUR_PER_DAY: f32 = 2.0;
pub const SAVE_LOSS_UNIT_PRICE: f32 = 4.0;
pub const TRANS_REPL_UNIT_PRICE: f32 = 150_000f32;
pub const TRANS_REPL_WITHIN_YEAR: f32 = 5.0;

pub const UNBAL_CALC_FACTOR: f32 = 1.0;
pub const REINVEST_RATE: f32 = 0.01;

//use sglib04::web1::ENERGY_GRW_RATE;

#[derive(Encode, Decode, Debug, Clone, Default)]
pub struct AojInfo {
    pub ar: String,
    pub level: Option<f32>,
    pub center_x: Option<f32>,
    pub center_y: Option<f32>,
    pub code: Option<String>,
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
    pub name: Option<String>,
    pub gons: Vec<Vec<(f32, f32)>>,
    pub pvid: String,
    pub sbids: HashSet<String>,
    pub fdids: HashSet<String>,
}

impl AojInfo {
    pub fn from(g: &GisAoj) -> AojInfo {
        AojInfo {
            ar: g.ar.clone(),
            level: g.level,
            center_x: g.center_x,
            center_y: g.center_y,
            code: g.code.clone(),
            sht_name: g.sht_name.clone(),
            shp_len: g.shp_len,
            office: g.office.clone(),
            parent1: g.parent1.clone(),
            parent2: g.parent2.clone(),
            pea: g.pea.clone(),
            ar_cd: g.ar_cd.clone(),
            shp_area: g.shp_area,
            prv_cd: g.prv_cd.clone(),
            aoj_sz: g.aoj_sz.clone(),
            reg: g.reg.clone(),
            name: g.name.clone(),
            gons: g.gons.clone(),
            ..Default::default()
        }
    }
}

/// ประมวลผลรวมเพื่อเกณฑ์การคัดเลือก
/// summery transformaters to substation
pub fn stage_03() -> Result<(), Box<dyn Error>> {
    println!("===== STAGE 3A =====");
    let buf = std::fs::read(format!("{DNM}/000_pea.bin")).unwrap();
    let (pea, _): (Pea, usize) =
        bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();
    let mut aids: Vec<_> = pea.aream.keys().collect();
    aids.sort();
    let subhs = p01_chk();
    //let sbtr = ld_sb_tr0();
    let sbtr = ld_sub_calc();
    //println!("sbtr: {}", sbtr.len());
    let mut emp = Vec::<(u32, f32)>::new();
    for y in OP_YEAR_START..=OP_YEAR_END {
        emp.push((y, 0f32));
    }
    let resc = re_scurv();
    //
    let mut m_aoj = HashMap::<String, AojInfo>::new();
    let mut v_pvas = Vec::<PeaAssVar>::new();
    let mut v_sbas = Vec::<PeaAssVar>::new();
    //let mut v_aojas = Vec::<PeaAssVar>::new();
    let mut m_aojas = HashMap::<String, PeaAssVar>::new();
    let mut m_sub = HashMap::<String, PeaSub>::new();
    let mut sbas_mx = PeaAssVar::from(0u64);
    for aid in aids {
        let Some(ar) = pea.aream.get(aid) else {
            continue;
        };
        println!("area {aid}");

        //======== AOJ CALCULATION PREP
        let eg = ProcEngine::prep6(aid);
        for aoj in &eg.aojs {
            let Some(ref aojcd) = aoj.code else {
                continue;
            };
            let aoj1 = AojInfo::from(aoj);
            m_aoj.insert(aojcd.clone(), aoj1);
            let mut ass = PeaAssVar::from(0u64);
            ass.aoj = aojcd.to_string();
            m_aojas.insert(ass.aoj.to_string(), ass);
        }
        let mut pids: Vec<_> = ar.provm.keys().collect();
        pids.sort();
        for pid in pids {
            let Some(prov) = ar.provm.get(pid) else {
                continue;
            };
            let mut pvas = PeaAssVar::from(0u64);
            pvas.arid = aid.to_string();
            pvas.pvid = pid.to_string();
            println!("  pv:{pid}");
            let mut sids: Vec<_> = prov.subm.keys().collect();
            sids.sort();
            for sid in sids {
                let Some(sb) = prov.subm.get(sid) else {
                    continue;
                };
                m_sub.insert(sid.clone(), sb.clone());
                // --- sub
                let Ok(buf) = std::fs::read(format!("{DNM}/{sid}.bin")) else {
                    //println!("PEA {sid} sub load error");
                    continue;
                };
                let (sb, _): (PeaSub, usize) =
                    bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();
                //println!("PEA SUB {sid} - {}", peasb.aojv.len());

                // --- sub row data 4
                let Ok(buf) = std::fs::read(format!("{DNM}/{sid}-rw4.bin")) else {
                    continue;
                };
                let (v_tras_raw, _): (Vec<PeaAssVar>, usize) =
                    bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();
                if v_tras_raw.is_empty() {
                    println!("    {sid} - NO data ");
                    continue;
                }
                let tras = &v_tras_raw[0];
                let mut sbas = PeaAssVar::from(0u64);
                sbas.arid = aid.to_string();
                sbas.pvid = pid.to_string();
                sbas.sbid = tras.sbid.to_string();
                sbas.n1d = sb.n1d_s;
                let note = if subhs.contains(&sbas.sbid) {
                    1f32
                } else {
                    0f32
                };

                let mut aojm = HashMap::<String, String>::new();
                for tras in &v_tras_raw {
                    sbas.add(tras);
                    for aoji in &tras.aojs {
                        let aoj = &eg.aojs[*aoji];
                        let aojcd = aoj.code.clone().unwrap().to_string();
                        if let Some(aojas) = m_aojas.get_mut(&aojcd) {
                            if aojas.pvid.is_empty() {
                                aojas.pvid = pid.to_string();
                            }
                            aojas.add(tras);
                        }
                        if let Some(aojif) = m_aoj.get_mut(&aojcd) {
                            aojif.sbids.insert(tras.sbid.to_string());
                            aojif.fdids.insert(tras.fdid.to_string());
                        }
                    }
                    let aoj = tras.aoj.clone();
                    aojm.entry(aoj.clone()).or_insert_with(|| aoj.clone());
                }
                sbas.v[VarType::EnGrowth.tousz()].v /= v_tras_raw.len() as f32;
                let mut aoj = String::new();
                for v in aojm.values() {
                    //for (_, v) in &m_aoj {
                    use std::fmt::Write;
                    if !aoj.is_empty() {
                        write!(aoj, ",").unwrap();
                    }
                    write!(aoj, "{}", v).unwrap();
                }

                // ============= SETTING GLOBAL
                sbas.aoj = "AOJ".to_string();
                sbas.aoj = aoj;
                sbas.aojv = sb.aojv.clone();
                sbas.copy(tras, VarType::NewCarReg);
                sbas.copy(tras, VarType::Gpp);
                sbas.copy(tras, VarType::MaxPosPowSub);
                sbas.copy(tras, VarType::MaxNegPowSub);
                sbas.copy(tras, VarType::VsppMv);
                sbas.copy(tras, VarType::SppHv);
                sbas.copy(tras, VarType::BigLotMv);
                sbas.copy(tras, VarType::BigLotHv);
                sbas.copy(tras, VarType::SubPowCap);
                sbas.copy(tras, VarType::SolarEnergy);
                sbas.copy(tras, VarType::PowTrSat);

                // ============= RECALCULATE START
                //let solar = sbas.v[VarType::SolarEnergy as usize].v;
                // re-calculation of value
                sbas.v[VarType::LvPowSatTr as usize].v =
                    sbas.v[VarType::PkPowTr as usize].v / z2o(sbas.v[VarType::PwCapTr as usize].v);
                sbas.v[VarType::CntLvPowSatTr as usize].v =
                    if sbas.v[VarType::LvPowSatTr as usize].v > 0.8f32 {
                        1f32
                    } else {
                        0f32
                    };
                sbas.v[VarType::ChgStnCap as usize].v = sbas.v[VarType::ChgStnCapTr as usize].v;
                sbas.v[VarType::ChgStnSell as usize].v = sbas.v[VarType::ChgStnSellTr as usize].v;
                sbas.v[VarType::MvPowSatTr as usize].v = sbas.v[VarType::MaxPosPowSub as usize].v
                    / z2o(sbas.v[VarType::SubPowCap as usize].v);
                sbas.v[VarType::MvVspp as usize].v = sbas.v[VarType::VsppMv as usize].v;
                sbas.v[VarType::HvSpp as usize].v = sbas.v[VarType::SppHv as usize].v;
                sbas.v[VarType::SmallSell as usize].v = sbas.v[VarType::SmallSellTr as usize].v;
                sbas.v[VarType::LargeSell as usize].v = sbas.v[VarType::LargeSellTr as usize].v;
                sbas.v[VarType::UnbalPow as usize].v = sbas.v[VarType::UnbalPowTr as usize].v;
                let v = sbas.v[VarType::UnbalPowTr as usize].v
                    / z2o(sbas.v[VarType::PwCapTr as usize].v);
                sbas.v[VarType::CntUnbalPow as usize].v = if v > 0.5f32 { 1f32 } else { 0f32 };
                // end of recalculation
                // ============= RECALCULATE END

                sbas.v[VarType::TakeNote as usize].v = note;
                sbas_mx.max(&sbas);
                let Some(_sbtr) = sbtr.get(&sbas.sbid) else {
                    continue;
                };

                // ==========  MV RENEW ENERGY
                //let engr = sbas.v[VarType::EnGrowth.tousz()].v;
                let pwmx = sbas.v[VarType::MaxPosPowSub.tousz()].v;
                for (i, rt) in resc.iter().enumerate() {
                    let rerev = if i < 3 {
                        0f32
                    } else {
                        rt * pwmx
                            * PEAK_POWER_RATIO
                            * RENEW_HOUR_PER_DAY
                            * 365.0
                            * RENEW_SAVE_PER_MWH
                    };
                    //let rerev = rt * pwrt * pwmx;
                    sbas.vy[VarType::FirMvReThb.tousz()].push(rerev);
                }
                sbas.sum_yr(VarType::FirMvReThb);
                //sbas.v[VarType::FirMvReThb.tousz()].v =
                //    sbas.vy[VarType::FirMvReThb.tousz()].iter().sum();

                // ==========  MV BATTERY
                let (mut sub, mut svg, mut dif, mut eng, _bescap) =
                    crate::ben2::ben_bess_calc(&sb, &sbas);
                let sub0 = sub[0] + sub[1] + sub[2];
                if sub0 > 0.0 {
                    println!("================= NOT OK =============== {sid}");
                }
                sbas.v[VarType::NoBess.tousz()].v = 1.0;
                sbas.vy[VarType::FirBatSubSave.tousz()].retain(|&_| false);
                sbas.vy[VarType::FirBatSvgSave.tousz()].retain(|&_| false);
                sbas.vy[VarType::FirBatEnerSave.tousz()].retain(|&_| false);
                sbas.vy[VarType::FirBatPriceDiff.tousz()].retain(|&_| false);

                sbas.v[VarType::FirBatSubSave.tousz()].v = sub.iter().sum();
                sbas.v[VarType::FirBatSvgSave.tousz()].v = svg.iter().sum();
                sbas.v[VarType::FirBatEnerSave.tousz()].v = eng.iter().sum();
                sbas.v[VarType::FirBatPriceDiff.tousz()].v = dif.iter().sum();

                sbas.vy[VarType::FirBatSubSave.tousz()].append(&mut sub);
                sbas.vy[VarType::FirBatSvgSave.tousz()].append(&mut svg);
                sbas.vy[VarType::FirBatEnerSave.tousz()].append(&mut eng);
                sbas.vy[VarType::FirBatPriceDiff.tousz()].append(&mut dif);

                ass_calc(&mut sbas)?;

                pvas.add(&sbas);
                pvas.copy(tras, VarType::NewCarReg);
                pvas.copy(tras, VarType::Gpp);

                v_sbas.push(sbas);
                //println!("   {sid} - {}", v_tras.len());
            } // end sub loop

            // check if already exists
            let pv = pvas.pvid.clone();
            let mut tmp = Vec::<PeaAssVar>::new();
            let mut add = Vec::<PeaAssVar>::new();
            tmp.append(&mut v_pvas);
            for a in tmp {
                if a.pvid == pv {
                    add.push(a);
                } else {
                    v_pvas.push(a);
                }
            }
            while let Some(a) = add.pop() {
                pvas.add(&a);
            }

            // re-calculation of value
            pvas.v[VarType::LvPowSatTr as usize].v =
                pvas.v[VarType::PkPowTr as usize].v / z2o(pvas.v[VarType::PwCapTr as usize].v);
            pvas.v[VarType::CntLvPowSatTr as usize].v =
                if pvas.v[VarType::LvPowSatTr as usize].v > 0.8f32 {
                    1f32
                } else {
                    0f32
                };
            pvas.v[VarType::ChgStnCap as usize].v = pvas.v[VarType::ChgStnCapTr as usize].v;
            pvas.v[VarType::ChgStnSell as usize].v = pvas.v[VarType::ChgStnSellTr as usize].v;
            pvas.v[VarType::MvPowSatTr as usize].v = pvas.v[VarType::MaxPosPowSub as usize].v
                / z2o(pvas.v[VarType::SubPowCap as usize].v);
            pvas.v[VarType::MvVspp as usize].v = pvas.v[VarType::VsppMv as usize].v;
            pvas.v[VarType::HvSpp as usize].v = pvas.v[VarType::SppHv as usize].v;
            pvas.v[VarType::SmallSell as usize].v = pvas.v[VarType::SmallSellTr as usize].v;
            pvas.v[VarType::LargeSell as usize].v = pvas.v[VarType::LargeSellTr as usize].v;
            pvas.v[VarType::UnbalPow as usize].v = pvas.v[VarType::UnbalPowTr as usize].v;
            let v =
                pvas.v[VarType::UnbalPowTr as usize].v / z2o(pvas.v[VarType::PwCapTr as usize].v);
            pvas.v[VarType::CntUnbalPow as usize].v = if v > 0.5f32 { 1f32 } else { 0f32 };
            // end of recalculation

            ass_calc(&mut pvas)?;

            v_pvas.push(pvas);
        } // end provi loop
    } // end area

    //========================================================
    //========================================================
    for (_cd, ass) in m_aojas.iter_mut() {
        ass.v[VarType::LvPowSatTr as usize].v =
            ass.v[VarType::PkPowTr as usize].v / z2o(ass.v[VarType::PwCapTr as usize].v);
        ass.v[VarType::CntLvPowSatTr as usize].v = if ass.v[VarType::LvPowSatTr as usize].v > 0.8f32
        {
            1f32
        } else {
            0f32
        };
        ass.v[VarType::ChgStnCap as usize].v = ass.v[VarType::ChgStnCapTr as usize].v;
        ass.v[VarType::ChgStnSell as usize].v = ass.v[VarType::ChgStnSellTr as usize].v;
        ass.v[VarType::MvPowSatTr as usize].v =
            ass.v[VarType::MaxPosPowSub as usize].v / z2o(ass.v[VarType::SubPowCap as usize].v);
        ass.v[VarType::MvVspp as usize].v = ass.v[VarType::VsppMv as usize].v;
        ass.v[VarType::HvSpp as usize].v = ass.v[VarType::SppHv as usize].v;
        ass.v[VarType::SmallSell as usize].v = ass.v[VarType::SmallSellTr as usize].v;
        ass.v[VarType::LargeSell as usize].v = ass.v[VarType::LargeSellTr as usize].v;
        ass.v[VarType::UnbalPow as usize].v = ass.v[VarType::UnbalPowTr as usize].v;
        let v = ass.v[VarType::UnbalPowTr as usize].v / z2o(ass.v[VarType::PwCapTr as usize].v);
        ass.v[VarType::CntUnbalPow as usize].v = if v > 0.5f32 { 1f32 } else { 0f32 };
        ass_calc(ass)?;
    }

    //////////////////////////////////////////////
    //  raking for substation
    for sbas in v_sbas.iter_mut() {
        let fir_cpx_opx = sbas.vy[VarType::FirCstRate.tousz()].clone();
        let guess = Some(0.);
        let fir: Vec<f64> = fir_cpx_opx.iter().map(|n| *n as f64).collect();
        let firr = financial::irr(&fir, guess).unwrap_or(0f64);
        sbas.v[VarType::FirCstRate.tousz()].v = firr as f32;
    }

    let mut uc1_v: Vec<_> = v_sbas
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc1Val as usize].v, i))
        .collect();
    uc1_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc1_v.iter().enumerate() {
        v_sbas[*i].v[VarType::Uc1Rank as usize].v = r as f32 + 1.0;
    }

    let mut uc2_v: Vec<_> = v_sbas
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc2Val as usize].v, i))
        .collect();
    uc2_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc2_v.iter().enumerate() {
        v_sbas[*i].v[VarType::Uc2Rank as usize].v = r as f32 + 1.0;
    }

    let mut uc3_v: Vec<_> = v_sbas
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc3Val as usize].v, i))
        .collect();
    uc3_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc3_v.iter().enumerate() {
        v_sbas[*i].v[VarType::Uc3Rank as usize].v = r as f32 + 1.0;
    }

    // save substation data
    let bin: Vec<u8> = bincode::encode_to_vec(&v_sbas, bincode::config::standard()).unwrap();
    std::fs::write(format!("{DNM}/000-sbrw.bin"), bin).unwrap();
    write_trn_ass_02(&v_sbas, &format!("{DNM}/000-sbrw0.txt"))?;
    //write_ass_csv_02(&v_sbas, &format!("{DNM}/000-sbrw0.csv"))?;

    //println!("SBAS MAX:{:?}", sbas_mx.v);
    let mut v_sbas_no = v_sbas.clone();
    for sub in v_sbas_no.iter_mut() {
        sub.nor(&sbas_mx);
    }
    let bin: Vec<u8> = bincode::encode_to_vec(&v_sbas_no, bincode::config::standard()).unwrap();
    std::fs::write(format!("{DNM}/000-sbno.bin"), bin).unwrap();
    write_trn_ass_02(&v_sbas_no, &format!("{DNM}/000-sbno0.txt"))?;

    //////////////////////////////////////////////
    //  raking for province
    for pvas in v_pvas.iter_mut() {
        let fir_cpx_opx = pvas.vy[VarType::FirCstRate.tousz()].clone();
        let guess = Some(0.);
        let fir: Vec<f64> = fir_cpx_opx.iter().map(|n| *n as f64).collect();
        let firr = financial::irr(&fir, guess).unwrap_or(0f64);
        pvas.v[VarType::FirCstRate.tousz()].v = firr as f32;
    }

    let mut uc1_v: Vec<_> = v_pvas
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc1Val as usize].v, i))
        .collect();
    uc1_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc1_v.iter().enumerate() {
        v_pvas[*i].v[VarType::Uc1Rank as usize].v = r as f32 + 1.0;
    }
    let mut uc2_v: Vec<_> = v_pvas
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc2Val as usize].v, i))
        .collect();
    uc2_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc2_v.iter().enumerate() {
        v_pvas[*i].v[VarType::Uc2Rank as usize].v = r as f32 + 1.0;
    }
    let mut uc3_v: Vec<_> = v_pvas
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc3Val as usize].v, i))
        .collect();
    uc3_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc3_v.iter().enumerate() {
        v_pvas[*i].v[VarType::Uc3Rank as usize].v = r as f32 + 1.0;
    }

    let bin: Vec<u8> = bincode::encode_to_vec(&v_pvas, bincode::config::standard()).unwrap();
    std::fs::write(format!("{DNM}/000-pvrw.bin"), bin).unwrap();

    //////////////////////////////////////////////
    //  raking for AOJs
    //let mut v_aojas: Vec<_> = m_aojas.into_iter().map(|(_, v)| v).collect();
    let mut v_aojas: Vec<_> = m_aojas.into_values().collect();

    for aojas in v_aojas.iter_mut() {
        let fir_cpx_opx = aojas.vy[VarType::FirCstRate.tousz()].clone();
        let guess = Some(0.);
        let fir: Vec<f64> = fir_cpx_opx.iter().map(|n| *n as f64).collect();
        let firr = financial::irr(&fir, guess).unwrap_or(0f64);
        aojas.v[VarType::FirCstRate.tousz()].v = firr as f32;
    }

    let mut uc1_v: Vec<_> = v_aojas
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc1Val as usize].v, i))
        .collect();
    uc1_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc1_v.iter().enumerate() {
        v_aojas[*i].v[VarType::Uc1Rank as usize].v = r as f32 + 1.0;
    }
    let mut uc2_v: Vec<_> = v_aojas
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc2Val as usize].v, i))
        .collect();
    uc2_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc2_v.iter().enumerate() {
        v_aojas[*i].v[VarType::Uc2Rank as usize].v = r as f32 + 1.0;
    }
    let mut uc3_v: Vec<_> = v_aojas
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc3Val as usize].v, i))
        .collect();
    uc3_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc3_v.iter().enumerate() {
        v_aojas[*i].v[VarType::Uc3Rank as usize].v = r as f32 + 1.0;
    }

    let bin: Vec<u8> = bincode::encode_to_vec(&v_aojas, bincode::config::standard()).unwrap();
    std::fs::write(format!("{DNM}/000-aojrw.bin"), bin).unwrap();

    let bin: Vec<u8> = bincode::encode_to_vec(&m_aoj, bincode::config::standard()).unwrap();
    let fnm = format!("{DNM}/000-aojm.bin");
    println!(" AOJM = {fnm}");
    std::fs::write(fnm, bin).unwrap();

    let bin: Vec<u8> = bincode::encode_to_vec(&m_sub, bincode::config::standard()).unwrap();
    let fnm = format!("{DNM}/000-subm.bin");
    println!(" SUBM = {fnm}");
    std::fs::write(fnm, bin).unwrap();

    Ok(())
}

/*
pub fn ass_calc(sbas: &mut PeaAssVar) -> Result<(), Box<dyn Error>> {
    // ==========  LOSS CALCULATION
    let unb_los = sbas.v[VarType::UnbalPowLossKw.tousz()].v
        * UNBAL_HOUR_PER_DAY
        * SAVE_LOSS_UNIT_PRICE
        * UNBAL_CALC_FACTOR
        * 365.0;
    //let unb_los = sbas.v[VarType::UnbalPowLossKw.tousz()].v * 4.0 * 4.0;
    //
    // claim save ratio 0.5
    let mut los_sav = unb_los * UNBAL_LOSS_CLAIM_RATE;
    //
    // transformer may die within 5 years
    // unit price for replace transformers
    // claim save ratio 0.5
    let mut tr_sav = sbas.v[VarType::CntTrSatLoss.tousz()].v / TRANS_REPL_WITHIN_YEAR
        * TRANS_REPL_UNIT_PRICE
        * TRANS_REPL_CLAIM_RATE;
    let mut ubt_sav = sbas.v[VarType::CntTrUnbalLoss.tousz()].v / TRANS_REPL_WITHIN_YEAR
        * TRANS_REPL_UNIT_PRICE
        * UNBAL_REPL_CLAIM_RATE;
    let mut all_sel = sbas.v[VarType::AllSellTr.tousz()].v
                    * NON_TECH_LOSS_RATIO
                    * 12.0 // in one year
                    * SAVE_LOSS_UNIT_PRICE
                    * NOTEC_LOSS_CLAIM_RATE;
    //sbas.v[VarType::AllSellTr.tousz()].v * 12.0 * 0.9 * 4_000f32 * 0.01;

    sbas.vy[VarType::FirUnbSave.tousz()].retain(|&_| false);
    sbas.vy[VarType::FirTrSatSave.tousz()].retain(|&_| false);
    sbas.vy[VarType::FirTrPhsSatSave.tousz()].retain(|&_| false);
    sbas.vy[VarType::FirNonTechLoss.tousz()].retain(|&_| false);
    for i in 0..15 {
        los_sav *= 1.0 + ENERGY_GRW_RATE;
        tr_sav *= 1.0 + ENERGY_GRW_RATE;
        ubt_sav *= 1.0 + ENERGY_GRW_RATE;
        all_sel *= 1.0 + ENERGY_GRW_RATE;
        //all_sel = 0.0;
        let (los, tr, ubt, all) = if i < 3 {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            (los_sav, tr_sav, ubt_sav, all_sel)
        };
        sbas.vy[VarType::FirUnbSave.tousz()].push(los);
        sbas.vy[VarType::FirTrSatSave.tousz()].push(tr);
        sbas.vy[VarType::FirTrPhsSatSave.tousz()].push(ubt);
        sbas.vy[VarType::FirNonTechLoss.tousz()].push(all);
    }
    //sbas.v[VarType::FirUnbSave.tousz()].v = sbas.vy[VarType::FirUnbSave.tousz()].iter().sum();
    sbas.sum_yr(VarType::FirUnbSave);
    //sbas.v[VarType::FirTrSatSave.tousz()].v = sbas.vy[VarType::FirTrSatSave.tousz()].iter().sum();
    sbas.sum_yr(VarType::FirTrSatSave);
    //sbas.v[VarType::FirTrPhsSatSave.tousz()].v =
    //    sbas.vy[VarType::FirTrPhsSatSave.tousz()].iter().sum();
    sbas.sum_yr(VarType::FirTrPhsSatSave);
    //sbas.v[VarType::FirNonTechLoss.tousz()].v =
    //    sbas.vy[VarType::FirNonTechLoss.tousz()].iter().sum();
    sbas.sum_yr(VarType::FirNonTechLoss);

    let mut fir_cpx_opx: Vec<f32> = vec![0f32; 15];
    let mut eir_cpx_opx: Vec<f32> = vec![0f32; 15];

    // CAPOPEX
    let mut capop: Vec<f32> = vec![0f32; 15];

    // CAPEX
    sbas.v[VarType::CstCapEx.tousz()].v = CAPEX_FLDS.iter().map(|vt| sbas.v[vt.tousz()].v).sum();
    let mut vy0: Vec<f32> = vec![0f32; 15];
    for vt in &CAPEX_FLDS {
        for (i, vy) in sbas.vy[vt.tousz()].iter().enumerate() {
            vy0[i] += vy;
            capop[i] += vy;
            fir_cpx_opx[i] -= vy;
            eir_cpx_opx[i] -= vy;
        }
    }
    sbas.vy[VarType::CstCapEx.tousz()] = vy0;

    let reinv = sbas.v[VarType::CstCapEx.tousz()].v * REINVEST_RATE;

    sbas.vy[VarType::CstReinvest.tousz()].retain(|&_| false);
    sbas.vy[VarType::CstReinvest.tousz()].append(&mut cst_reinvest(reinv));
    //sbas.v[VarType::CstReinvest.tousz()].v = sbas.vy[VarType::CstReinvest.tousz()].iter().sum();
    sbas.sum_yr(VarType::CstReinvest);

    // OPEX
    sbas.v[VarType::CstOpEx.tousz()].v = OPEX_FLDS.iter().map(|vt| sbas.v[vt.tousz()].v).sum();
    let mut vy0: Vec<f32> = vec![0f32; 15];
    for vt in &OPEX_FLDS {
        for (i, vy) in sbas.vy[vt.tousz()].iter().enumerate() {
            vy0[i] += vy;
            capop[i] += vy;
            fir_cpx_opx[i] -= vy;
            eir_cpx_opx[i] -= vy;
        }
    }
    sbas.vy[VarType::CstOpEx.tousz()] = vy0;

    sbas.v[VarType::CstCapOpEx.tousz()].v = sbas.v[VarType::CstOpEx.tousz()].v
        + sbas.v[VarType::CstCapEx.tousz()].v
        + sbas.v[VarType::CstReinvest.tousz()].v;
    sbas.vy[VarType::CstCapOpEx.tousz()] = capop;

    // FIR
    sbas.v[VarType::FirSum.tousz()].v = FIR_FLDS.iter().map(|vt| sbas.v[vt.tousz()].v).sum();
    let mut vy0: Vec<f32> = vec![0f32; 15];
    for vt in &FIR_FLDS {
        if sbas.vy[vt.tousz()].len() > 15 {
            //println!("exceed {vt:?} = {}", sbas.vy[vt.tousz()].len());
        }
        for (i, vy) in sbas.vy[vt.tousz()].iter().take(15).enumerate() {
            vy0[i] += vy;
            fir_cpx_opx[i] += vy;
        }
    }
    sbas.vy[VarType::FirSum.tousz()] = vy0;

    // EIR
    sbas.v[VarType::EirSum.tousz()].v = EIR_FLDS.iter().map(|vt| sbas.v[vt.tousz()].v).sum();
    let mut vy0: Vec<f32> = vec![0f32; 15];
    for vt in &EIR_FLDS {
        for (i, vy) in sbas.vy[vt.tousz()].iter().enumerate() {
            vy0[i] += vy;
            eir_cpx_opx[i] -= vy;
        }
    }
    sbas.vy[VarType::EirSum.tousz()] = vy0;

    let guess = Some(0.);
    let fir: Vec<f64> = fir_cpx_opx.iter().map(|n| *n as f64).collect();
    let firr = financial::irr(&fir, guess).unwrap_or(0f64);
    let firr = firr * 100.0;
    let eir: Vec<f64> = eir_cpx_opx.iter().map(|n| *n as f64).collect();
    let eirr = financial::irr(&eir, guess).unwrap_or(0f64);
    let eirr = eirr * 100.0;
    //println!("FIRR: {}", firr);

    sbas.vy[VarType::FirCstRate.tousz()] = fir_cpx_opx;
    sbas.vy[VarType::EirCstRate.tousz()] = eir_cpx_opx;
    sbas.v[VarType::FirCstRate.tousz()].v = firr as f32;
    sbas.v[VarType::EirCstRate.tousz()].v = eirr as f32;

    Ok(())
}
*/
