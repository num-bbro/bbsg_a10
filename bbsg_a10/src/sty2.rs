use crate::asm::ASM::*;
use crate::dcl::ProcEngine;
use crate::dcl::*;
use crate::p08::p08_class_val;
use crate::p08::ProfType;
use crate::utl::mon_kwh_2_kw;
use crate::utl::trf_kva_2_kw;
use crate::utl::*;
use crate::utl6::Assumption;
use regex::Regex;
use sglib04::geo4::PowerProdType;
use std::collections::HashMap;
use std::error::Error;

use crate::cst3::cst_bes_imp;
use crate::cst3::cst_bes_ins;
use crate::cst3::cst_bes_op;
use crate::cst3::cst_comm_imp;
use crate::cst3::cst_comm_ins;
use crate::cst3::cst_comm_op;
use crate::cst3::cst_m1p_imp;
use crate::cst3::cst_m1p_ins;
use crate::cst3::cst_m1p_op;
use crate::cst3::cst_m3p_imp;
use crate::cst3::cst_m3p_ins;
use crate::cst3::cst_m3p_op;
use crate::cst3::cst_plfm_imp;
use crate::cst3::cst_plfm_ins;
use crate::cst3::cst_plfm_op;
use crate::cst3::cst_reinvest;
use crate::cst3::cst_tr_imp;
use crate::cst3::cst_tr_ins;
use crate::cst3::cst_tr_op;
use crate::cst3::eir_cust_etruck_save;
use crate::cst3::eir_cust_ev_save;
use crate::cst3::eir_cust_loss_save;
use crate::cst3::eir_cust_mv_rev;
use crate::cst3::eir_cust_save;
use crate::cst3::eir_cust_solar_roof;
use crate::cst3::eir_en_rev_save;
use crate::cst3::eir_ghg_save;
use std::sync::Arc;
use std::sync::Mutex;

//pub const BESS_EVPOW_MWH_MULT: f32 = 1.0;
use std::sync::mpsc;
use std::thread;
use crate::utl7::get_brn_map;

pub const PRV_LEVEL_FLDS: [usize; 2] = [VarType::NewCarReg as usize, VarType::Gpp as usize];

pub const SUB_LEVEL_FLDS: [usize; 11] = [
    VarType::NewCarReg as usize,
    VarType::Gpp as usize,
    VarType::MaxPosPowSub as usize,
    VarType::MaxNegPowSub as usize,
    VarType::VsppMv as usize,
    VarType::SppHv as usize,
    VarType::BigLotMv as usize,
    VarType::BigLotHv as usize,
    VarType::SubPowCap as usize,
    VarType::SolarEnergy as usize,
    VarType::PowTrSat as usize,
];

pub const FEED_LEVEL_FLDS: [usize; 13] = [
    VarType::NewCarReg as usize,
    VarType::Gpp as usize,
    VarType::MaxPosPowSub as usize,
    VarType::MaxNegPowSub as usize,
    VarType::VsppMv as usize,
    VarType::SppHv as usize,
    VarType::BigLotMv as usize,
    VarType::BigLotHv as usize,
    VarType::SubPowCap as usize,
    VarType::SolarEnergy as usize,
    VarType::PowTrSat as usize,
    VarType::MaxPosPowFeeder as usize,
    VarType::MaxNegPowFeeder as usize,
];

use crate::utl4::NumValEnum;
use serde_json::Value;

// read 000_pea.bin
// read SSS.bin
/*
pub fn stage_02(coreno: usize, vwid: String) -> Result<(), Box<dyn Error>> {
    println!("===== STAGE 2 =====");
    let ac = crate::utl4::make_archi(&vwid)?;
    let dnm = ac.t(OUTDIR);
    crate::dcl::set_dirnm(&dnm);

    //let mut tras_mx1 = PeaAssVar::from(0u64);
    //let mut assrw1 = Vec::<PeaAssVar>::new();
    //let (assrw1,mx1) = stage_02_1(coreno, &ac, &mut assrw1, &mut tras_mx1)?;
    let asrw = stage_02_1(coreno, &ac)?;
    println!("asrw: {}", asrw.len());
    stage_02_b(coreno, &ac, asrw)?;
    Ok(())
}
*/

use crate::utl6::ArchiInfo;

pub fn stage_02_1(_coreno: usize, arif: &ArchiInfo) -> Result<Vec<PeaAssVar>, Box<dyn Error>> {
    let ass = arif.assumption();
    let assv = Vec::<PeaAssVar>::new();
    let tik = std::time::SystemTime::now();
    let e0 = ProcEngine::prep5();

    //===== LOAD PROF FD2FD
    let keys: Vec<_> = e0.lp24.keys().collect();
    let re = Regex::new(r"([A-Z]{3})-([0-9]{2})[VW].*").unwrap();
    let mut fd2fd = HashMap::<String, String>::new();
    for k in keys {
        for cap in re.captures_iter(k) {
            let a = &cap[1].to_string();
            let b = &cap[2].to_string();
            let fd = format!("{a}{b}");
            if let Some(_o) = fd2fd.get(&fd) {
                //println!("DUP {o} => fd:{fd} k:{k}");
            } else {
                fd2fd.insert(fd, k.to_string());
            }
        }
    }

    //===== LOAD 000_PEA
    let a_fd2fd = Arc::new(Mutex::new(fd2fd));
    let a_e0 = Arc::new(Mutex::new(e0));
    let a_assv = Arc::new(Mutex::new(assv));

    let dnm = ass.t(OUTDIR);
    let fnm = format!("{dnm}/000_pea.bin");
    println!("fnm:{fnm}");
    let buf = std::fs::read(fnm).unwrap();
    let (mut pea, _): (Pea, usize) =
        bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();

    // =========== ASSUMPTION SETTING ============
    // =========== ASSUMPTION SETTING ============
    // =========== ASSUMPTION SETTING ============
    let ass = arif.assumption();
    let grw0 = ass.v(EN_AVG_GRW_RATE);
    let trf_loss_ratio0 = ass.v(TRF_LOSS_RATIO);
    let trf_unbal_k0 = ass.v(TRF_UNBAL_K);
    let trf_unbal_cnt_rate0 = ass.v(TRF_UNBAL_CNT_RATE);

    //let ac = ac.clone();
    //===== PROCESS EACH AREA
    std::thread::scope(|s| {
        let ass = ass.clone();
        for (aid, ar) in pea.aream.iter_mut() {
            //
            // ========= ASSUMPTION CLONE ==========
            // ========= ASSUMPTION CLONE ==========
            // ========= ASSUMPTION CLONE ==========
            let grwz = grw0;
            let mut grwm = grw0;
            let trf_loss_ratio = trf_loss_ratio0;
            let trf_unbal_k = trf_unbal_k0;
            let trf_unbal_cnt_rate = trf_unbal_cnt_rate0;

            //for id in aids {
            let id = aid.to_string();
            //let ac = ac.clone();

            let c_e0 = a_e0.clone();
            let c_fd2fd = a_fd2fd.clone();
            let c_assv = a_assv.clone();
            let dnm = dnm.clone();
            let ass = ass.clone();

            //let id = id.to_string();
            //let aream = pea.aream.clone();
            let _handle = s.spawn(move || {
                let ass = ass.clone();
                let eg = ProcEngine::prep3(&id);
                let mut am_dn = HashMap::<String, (f32, f32)>::new();
                for dn in &eg.amps {
                    let key = dn.key.to_string();
                    if let Some((_po, aa)) = am_dn.get_mut(&key) {
                        *aa += dn.area;
                    } else {
                        am_dn.insert(key, (dn.popu, dn.area));
                    }
                }
                //==== AMPHO INIT
                //--- municipality initialization
                //==== MUNI INIT
                let mut mu_dn = HashMap::<String, (f32, f32)>::new();
                for dn in &eg.muni {
                    let key = dn.key.to_string();
                    if let Some((_po, aa)) = mu_dn.get_mut(&key) {
                        *aa += dn.area;
                    } else {
                        mu_dn.insert(key, (dn.popu, dn.area));
                    }
                }
                let mut pids: Vec<_> = ar.provm.keys().collect();
                pids.sort();
                //
                //==== PROV LOOP
                for pid in pids {
                    let Some(prov) = ar.provm.get(pid) else {
                        continue;
                    };
                    let vp01 = prov.evpc;
                    let vp02 = prov.gppv;
                    //let evlk = *EV_LIKELY.get(pid).unwrap_or(&0f32);
                    //let selk = *SELE_LIKELY.get(pid).unwrap_or(&0f32);
                    //println!("  pv:{pid}");
                    let mut sids: Vec<_> = prov.subm.keys().collect();
                    sids.sort();
                    //==== SUBST LOOP
                    for sid in sids {
                        // check if the substation exists
                        let Some(_sb) = prov.subm.get(sid) else {
                            continue;
                        };
                        // read substation data from storage
                        let Ok(buf) = std::fs::read(format!("{dnm}/{sid}.bin")) else {
                            continue;
                        };
                        let (sub, _): (PeaSub, usize) =
                            bincode::decode_from_slice(&buf[..], bincode::config::standard())
                                .unwrap();
                        // Substation
                        //=====================================================
                        //=====================================================
                        let mut vs01 = 0f32;
                        let mut vs02 = 0f32;
                        let mut solar = 0f32;
                        let mut sopeek = 0f32;
                        //==== MAX POWER
                        if let Ok(e0) = c_e0.lock() {
                            if let Some(lp) = e0.lp24.get(sid) {
                                for v in lp.pos_rep.val.into_iter().flatten() {
                                    vs01 = vs01.max(v.unwrap_or(0f32));
                                }
                            } else if let Some(lp) = e0.lp23.get(sid) {
                                for v in lp.pos_rep.val.into_iter().flatten() {
                                    vs01 = vs01.max(v.unwrap_or(0f32));
                                }
                            };
                            if let Some(lp) = e0.lp24.get(sid) {
                                for v in lp.neg_rep.val.into_iter().flatten() {
                                    vs02 = vs02.max(v.unwrap_or(0f32));
                                }
                            } else if let Some(lp) = e0.lp23.get(sid) {
                                for v in lp.neg_rep.val.into_iter().flatten() {
                                    vs02 = vs02.max(v.unwrap_or(0f32));
                                }
                            }
                            if let Some(lp) = e0.lp24.get(sid) {
                                if let Some(lpv) = &lp.pos_rep.val {
                                    if let Ok(lpf) = p08_class_val(lpv, &ass) {
                                        if lpf.lp_type == ProfType::SolarPower
                                            || lpf.lp_type == ProfType::SolarNight
                                        {
                                            solar = -lpf.sol_en.unwrap_or(0f32);
                                            sopeek = -lpf.sol_pk.unwrap_or(0f32);
                                        }
                                    }
                                }
                            }
                        }
                        //====  DER: VSPP SPP
                        let mut vs03 = 0f32;
                        for pi in &sub.vspps {
                            vs03 += eg.vsps[*pi].kw.unwrap_or(0f32);
                        }
                        let mut vs04 = 0f32;
                        for pi in &sub.spps {
                            vs04 += eg.spps[*pi].mw.unwrap_or(0f32);
                        }
                        let mut vs05 = 0f32;
                        for pi in &sub.repls {
                            if let PowerProdType::SPP = eg.repl[*pi].pptp {
                                vs05 += eg.repl[*pi].pwmw.unwrap_or(0f32);
                            }
                        }
                        let mut vs06 = 0f32;
                        for pi in &sub.repls {
                            if let PowerProdType::VSPP = eg.repl[*pi].pptp {
                                vs06 += eg.repl[*pi].pwmw.unwrap_or(0f32);
                            }
                        }
                        let vs07 = sub.mvxn as f32;

                        //println!(" pv:{pid} sb:{sid} feed: {}", sub.feeders.len());
                        //==== FEEDER LOOP
                        let mut fids: Vec<_> = sub.feedm.keys().collect();
                        fids.sort();
                        let mut s_tr_ass = Vec::<PeaAssVar>::new();
                        for fid in fids {
                            let Some(fd) = sub.feedm.get(fid) else {
                                continue;
                            };
                            ////////////////////////////////////////////////////////
                            ////////////////////////////////////////////////////////
                            // Feeder
                            //let k1 = format!("{fid}");
                            let k1 = fid.to_string();
                            let key = if let Ok(fd2fd) = c_fd2fd.lock() {
                                fd2fd.get(&k1).unwrap_or(&"-".to_string()).to_string()
                            } else {
                                "".to_string()
                            };
                            //let mut grw = ac.v(EN_AVG_GRW_RATE);
                            let (mut af01, mut af03, mut af02, mut af04) = (None, None, None, None);

                            //==== LOAD PROFILE CALC
                            if let Ok(e0) = c_e0.lock() {
                                if let (Some(lp1), Some(lp0)) =
                                    (e0.lp24.get(&key), e0.lp23.get(&key))
                                {
                                    let mut pwmx = 0f32;
                                    if let Some(reps) = lp1.pos_rep.val {
                                        for vv in reps.iter().flatten() {
                                            pwmx = pwmx.max(*vv);
                                        }
                                    };
                                    let mut pwmx0 = 0f32;
                                    if let Some(reps) = lp0.pos_rep.val {
                                        for vv in reps.iter().flatten() {
                                            pwmx0 = pwmx0.max(*vv);
                                        }
                                    };

                                    let grw2 = if pwmx0 > 0f32 {
                                        (pwmx - pwmx0) / pwmx * 100f32
                                    } else {
                                        0f32
                                    };
                                    if grw2 > grwm && grw2 < grwz {
                                        grwm = grw2;
                                    }
                                }

                                if let Some(lp) = e0.lp24.get(&key) {
                                    if let Some(vv) = lp.pos_rep.val {
                                        for v in vv.iter().flatten() {
                                            if let Some(v0) = af01 {
                                                af01 = Some(v.max(v0))
                                            } else {
                                                af01 = Some(*v);
                                            }
                                            if let Some(v0) = af03 {
                                                af03 = Some(v.min(v0))
                                            } else {
                                                af03 = Some(*v);
                                            }
                                        }
                                    }
                                    if let Some(vv) = lp.neg_rep.val {
                                        for v in vv.iter().flatten() {
                                            if let Some(v0) = af02 {
                                                af02 = Some(v.max(v0))
                                            } else {
                                                af02 = Some(*v);
                                            }
                                            if let Some(v0) = af04 {
                                                af04 = Some(v.min(v0))
                                            } else {
                                                af04 = Some(*v);
                                            }
                                        }
                                    }
                                } else if let Some(lp) = e0.lp23.get(&key) {
                                    if let Some(vv) = lp.pos_rep.val {
                                        for v in vv.iter().flatten() {
                                            if let Some(v0) = af01 {
                                                af01 = Some(v.max(v0))
                                            } else {
                                                af01 = Some(*v);
                                            }
                                            if let Some(v0) = af03 {
                                                af03 = Some(v.min(v0))
                                            } else {
                                                af03 = Some(*v);
                                            }
                                        }
                                    }
                                    if let Some(vv) = lp.neg_rep.val {
                                        for v in vv.iter().flatten() {
                                            if let Some(v0) = af02 {
                                                af02 = Some(v.max(v0))
                                            } else {
                                                af02 = Some(*v);
                                            }
                                            if let Some(v0) = af04 {
                                                af04 = Some(v.min(v0))
                                            } else {
                                                af04 = Some(*v);
                                            }
                                        }
                                    }
                                };
                            }
                            let vf01 = af01.unwrap_or(0f32);
                            let mut vf03 = af03.unwrap_or(0f32);
                            vf03 = vf01 - vf03;
                            let vf02 = af02.unwrap_or(0f32);
                            let mut vf04 = af04.unwrap_or(0f32);
                            vf04 = vf02 - vf04;

                            let mut tids: Vec<_> = fd.tranm.keys().collect();
                            tids.sort();

                            // =========================
                            // loop on each transformer
                            //==== TRANS LOOP
                            for tid in tids {
                                let Some(trn) = fd.tranm.get(tid) else {
                                    continue;
                                };
                                let aojs = trn.aojs.clone();
                                let vt05 = trn.tr_kva.unwrap_or(10f32);
                                let vt05 = trf_kva_2_kw(vt05);
                                let mut vt06 = 1f32;
                                //==== ZONE PROCESS
                                for zi in &trn.zons {
                                    match eg.zons[*zi].zncd.clone().expect("-").as_str() {
                                        "21" | "22" | "24" => {
                                            vt06 = vt06.max(5f32);
                                        }
                                        "11" | "12" | "13" | "14" => {
                                            vt06 = vt06.max(4f32);
                                        }
                                        "23" | "25" | "31" => {
                                            vt06 = vt06.max(3f32);
                                        }
                                        "41" | "42" => {
                                            vt06 = vt06.max(2f32);
                                        }
                                        _ => {}
                                    }
                                }
                                //===== ZONE for TPA
                                let mut z_tpa = 0f32;
                                for zi in &trn.zons {
                                    match eg.zons[*zi].zncd.clone().expect("-").as_str() {
                                        "11" | "12" | "13" | "14" => {
                                            z_tpa = vt06.max(5f32);
                                        }
                                        "21" | "22" | "23" | "24" | "25" | "31" | "41" | "42" => {
                                            z_tpa = vt06.max(1f32);
                                        }
                                        "51" => {
                                            z_tpa = vt06.max(3f32);
                                        }
                                        _ => {}
                                    }
                                }

                                let (aoj, aojcd) = if aojs.is_empty() {
                                    ("-".to_string(), "-".to_string())
                                } else {
                                    let ai = aojs[0];
                                    let aoj =
                                        eg.aojs[ai].sht_name.clone().unwrap_or("-".to_string());
                                    let aojcd = eg.aojs[ai].code.clone().unwrap_or("-".to_string());
                                    (aoj, aojcd)
                                };

                                let mut vt01 = 0f32;
                                let mut vt02 = 0f32;
                                let mut vt10 = 0f32;
                                let mut nom1p = 0f32;
                                let mut nom3p = 0f32;
                                let mut allsel = 0f32;

                                let (mut se_a, mut se_b, mut se_c) = (0.0, 0.0, 0.0);
                                let (mut sl_a, mut sl_b, mut sl_c, mut sl_3) = (0.0, 0.0, 0.0, 0.0);
                                //
                                // =========================
                                // loop on each meter
                                //==== METER LOOP
                                for met in &trn.mets {
                                    ///////////////////////////////////////////////////
                                    ///////////////////////////////////////////////////
                                    // Meter
                                    //==== METER TYPE
                                    if let MeterAccType::Small = met.met_type {
                                        if met.main.is_empty() && met.kwh18 > 600f32 {
                                            //if met.main.is_empty() && met.kwh18 > 200f32 {
                                            vt01 += 1f32;
                                            vt02 += met.kwh15;
                                        }
                                        allsel += met.kwh15;
                                    } else if let MeterAccType::Large = met.met_type {
                                        vt10 += met.kwh15;
                                        print!(">>>> LARGE >>>> _{}", met.kwh15);
                                        //allsel += met.kwh15;
                                    }
                                    //==== METER PHASE
                                    if trn.own == "P" {
                                        match met.mt_phs.clone().unwrap_or(String::new()).as_str() {
                                            "A" => se_a += met.kwh15,
                                            "B" => se_b += met.kwh15,
                                            "C" => se_c += met.kwh15,
                                            _ => {}
                                        }
                                        match met.mt_phs.clone().unwrap_or(String::new()).as_str() {
                                            "A" => sl_a += met.kwh15,
                                            "B" => sl_b += met.kwh15,
                                            "C" => sl_c += met.kwh15,
                                            _ => sl_3 += met.kwh15,
                                        }
                                        match met.mt_phs.clone().unwrap_or(String::new()).as_str() {
                                            "A" | "B" | "C" => nom1p += 1.0,
                                            _ => nom3p += 1.0,
                                        }
                                    }
                                }
                                let vt11 = trn.mets.len() as f32;
                                let vt12 = 1f32;
                                sl_3 = mon_kwh_2_kw(sl_3);
                                sl_a = mon_kwh_2_kw(sl_a);
                                sl_b = mon_kwh_2_kw(sl_b);
                                sl_c = mon_kwh_2_kw(sl_c);
                                let v_phs_a = sl_3 + sl_a;
                                let v_phs_b = sl_3 + sl_b;
                                let v_phs_c = sl_3 + sl_c;
                                let v_all_p = sl_3 + sl_a + sl_b + sl_c;
                                let v_ph_av = (v_phs_a + v_phs_b + v_phs_c) / 3f32;
                                let v_ph_mx = v_phs_a.max(v_phs_b.max(v_phs_c));
                                let v_ph_rt = v_ph_mx / z2o(v_ph_av);
                                let v_al_kw = v_phs_a + v_phs_b + v_phs_c;
                                let v_loss = v_al_kw * trf_loss_ratio;
                                let v_unba = v_loss * trf_unbal_k * v_ph_rt * v_ph_rt;
                                let v_unb_sat = v_ph_mx / z2o(vt05);
                                let v_unb_cnt = if v_unb_sat >= trf_unbal_cnt_rate {
                                    1f32
                                } else {
                                    0f32
                                };
                                let v_max_sat = v_all_p / z2o(vt05);
                                let v_max_cnt =
                                    if v_unb_cnt == 0f32 && v_max_sat >= trf_unbal_cnt_rate {
                                        1f32
                                    } else {
                                        0f32
                                    };
                                let mut vt08 = 0f32;
                                let se_p = se_a + se_b + se_c;
                                if se_a < se_p && se_b < se_p && se_c < se_p {
                                    let ab = (se_a - se_b).abs();
                                    let bc = (se_b - se_c).abs();
                                    let ca = (se_c - se_a).abs();
                                    vt08 = (ab + bc + ca) * 0.5;
                                }
                                let vt08 = mon_kwh_2_kw(vt08);
                                //let vt09 = trf_kva_2_kw(vt02);
                                let vt09 = mon_kwh_2_kw(vt02);

                                let mut vt03 = 0f32;
                                for vi in &trn.vols {
                                    for (pw, no) in &eg.vols[*vi].chgr {
                                        vt03 += (*pw * *no) as f32;
                                    }
                                }
                                let mut vt04 = 0f32;
                                for vi in &trn.vols {
                                    for (_yr, am) in &eg.vols[*vi].sell {
                                        vt04 += *am;
                                    }
                                }
                                let mut vt07 = 1f32;
                                for ai in &trn.amps {
                                    let am = &eg.amps[*ai].key;
                                    if let Some((p, a)) = am_dn.get(am) {
                                        let a = a / 1_000f32;
                                        let pd = p / a * 0.6f32;
                                        let v = match pd {
                                            0f32..30f32 => 1f32,
                                            30f32..60f32 => 2f32,
                                            60f32..150f32 => 3f32,
                                            150f32..500f32 => 4f32,
                                            _ => 5f32,
                                        };
                                        vt07 = vt07.max(v);
                                    }
                                }
                                for ai in &trn.muns {
                                    let mu = &eg.muni[*ai].key;
                                    if let Some((p, a)) = mu_dn.get(mu) {
                                        let a = a / 1_000f32;
                                        let pd = p / a * 2.5f32;
                                        let v = match pd {
                                            0f32..15f32 => 6f32,
                                            15f32..30f32 => 7f32,
                                            30f32..70f32 => 8f32,
                                            70f32..200f32 => 9f32,
                                            _ => 10f32,
                                        };
                                        vt07 = vt07.max(v);
                                    }
                                }

                                // transformer data finish
                                let mut tr_as = PeaAssVar::from(trn.n1d);
                                tr_as.arid = aid.to_string();
                                tr_as.pvid = pid.to_string();
                                tr_as.sbid = sid.to_string();
                                tr_as.fdid = fid.to_string();
                                tr_as.own = trn.own.to_string();
                                tr_as.peano =
                                    trn.tr_pea.clone().unwrap_or("".to_string()).to_string();
                                tr_as.aoj = aoj;
                                tr_as.aojcd = aojcd;
                                tr_as.aojs = aojs;
                                tr_as.v[VarType::None as usize].v = 0f32;
                                tr_as.v[VarType::NewCarReg as usize].v = vp01;
                                tr_as.v[VarType::Gpp as usize].v = vp02;
                                tr_as.v[VarType::MaxPosPowSub as usize].v = vs01;
                                tr_as.v[VarType::MaxNegPowSub as usize].v = vs02;
                                tr_as.v[VarType::VsppMv as usize].v = vs03;
                                tr_as.v[VarType::SppHv as usize].v = vs04;
                                tr_as.v[VarType::BigLotMv as usize].v = vs05;
                                tr_as.v[VarType::BigLotHv as usize].v = vs06;
                                tr_as.v[VarType::SubPowCap as usize].v = vs07;
                                tr_as.v[VarType::MaxPosPowFeeder as usize].v = vf01;
                                let pow_tr_sat = vs01 / trf_kva_2_kw(z2o(vs07));
                                tr_as.v[VarType::PowTrSat as usize].v = pow_tr_sat;

                                tr_as.v[VarType::MaxNegPowFeeder as usize].v = vf02;
                                tr_as.v[VarType::MaxPosDiffFeeder as usize].v = vf03;
                                tr_as.v[VarType::MaxNegDiffFeeder as usize].v = vf04;
                                tr_as.v[VarType::NoMeterTrans as usize].v = vt01;
                                tr_as.v[VarType::SmallSellTr as usize].v = vt02;
                                tr_as.v[VarType::ChgStnCapTr as usize].v = vt03;
                                tr_as.v[VarType::ChgStnSellTr as usize].v = vt04;
                                tr_as.v[VarType::PwCapTr as usize].v = vt05;
                                tr_as.v[VarType::ZoneTr as usize].v = vt06;
                                tr_as.v[VarType::PopTr as usize].v = vt07;
                                tr_as.v[VarType::UnbalPowTr as usize].v = vt08;
                                tr_as.v[VarType::PkPowTr as usize].v = vt09;
                                tr_as.v[VarType::LargeSellTr as usize].v = vt10;
                                tr_as.v[VarType::AllNoMeterTr as usize].v = vt11;
                                tr_as.v[VarType::NoMet1Ph as usize].v = nom1p;
                                tr_as.v[VarType::NoMet3Ph as usize].v = nom3p;
                                tr_as.v[VarType::NoTr as usize].v = vt12;
                                tr_as.v[VarType::EnGrowth as usize].v = grwm;
                                if trn.own == "P" {
                                    tr_as.v[VarType::NoPeaTr as usize].v = vt12;
                                } else {
                                    tr_as.v[VarType::NoCusTr as usize].v = vt12;
                                }
                                tr_as.v[VarType::PkSelPowPhsAKw as usize].v = v_phs_a;
                                tr_as.v[VarType::PkSelPowPhsBKw as usize].v = v_phs_b;
                                tr_as.v[VarType::PkSelPowPhsCKw as usize].v = v_phs_c;
                                tr_as.v[VarType::PkSelPowPhsAvg as usize].v = v_ph_av;
                                tr_as.v[VarType::PkSelPowPhsMax as usize].v = v_ph_mx;
                                tr_as.v[VarType::UnbalPowRate as usize].v = v_ph_rt;
                                tr_as.v[VarType::TransLossKw as usize].v = v_loss;
                                tr_as.v[VarType::UnbalPowLossKw as usize].v = v_unba;
                                tr_as.v[VarType::CntTrUnbalLoss as usize].v = v_unb_cnt;
                                tr_as.v[VarType::CntTrSatLoss as usize].v = v_max_cnt;
                                //tr_as.v[VarType::EvCarLikely as usize].v = evlk;
                                //tr_as.v[VarType::SelectLikely as usize].v = selk;
                                tr_as.v[VarType::SolarEnergy as usize].v = solar;
                                tr_as.v[VarType::SubSolarPeekMw as usize].v = sopeek;
                                tr_as.v[VarType::AllSellTr.tousz()].v = allsel;

                                tr_as.v[VarType::TpaZone.tousz()].v = z_tpa;

                                //tr_as.v[VarType::OfficeCovWg.tousz()].v = aojs.len();
                                //s_tr_sum.add(&tr_as);
                                s_tr_ass.push(tr_as);
                            } // end trans loop
                        } // end feeder loop
                        if let Ok(mut assv) = c_assv.lock() {
                            assv.append(&mut s_tr_ass);
                        }
                    } // end sub loop
                } // end provi loop

                //let mut aoj_m = HashMap::<usize, usize>::new();
                ////////////////////////////////
                ////////////////////////////////
            });
        } // end area
    });
    let se = tik.elapsed().unwrap().as_secs();
    println!("======== RAW1  - {se} sec");

    if let Ok(assv) = a_assv.lock() {
        return Ok(assv.to_vec());
    }
    Err("ERROR".into())
}

/*
pub fn stage_02_b(
    coreno: usize,
    arif: &ArchiInfo,
    assrw1: Vec<PeaAssVar>,
) -> Result<(), Box<dyn Error>> {
    //  Assumption Constants
    let ass = arif.assumption();
    let me_x = arif.ass.v(METER_NO_MULTIPLY);
    let bess_x = arif.ass.v(BESS_EVCAP_MULTIPLY);

    println!("============ me_x: {me_x} bess_x : {bess_x}");

    //let cn = 10;
    //let sz = (assrw1.len() + cn - 1) / cn;
    let sz = assrw1.len().div_ceil(coreno);
    let dnm = arif.ass.t(OUTDIR);

    /*
    let tpo_no = 20_000;
    let ecu_no = 10_000;
    let svg_no = 300;
     */

    let tpo_no = arif.ass.v(TPO_DEV_NO_REQ);
    let ecu_no = arif.ass.v(ECU_DEV_NO_REQ);
    //println!("===== TPO: {tpo_no}");
    //println!("===== ECU: {ecu_no}");

    let tpo_no = tpo_no as usize;
    let ecu_no = ecu_no as usize;
    /*
    let svg_no = arif.ass.v(SVG_DEV_NO_REQ);
    let svg_no = svg_no as usize;
    println!("===== SVG: {svg_no}");
    */

    //////////////////////////////////////////////
    // EV Weight
    let mut we_ev = PeaAssVar::from(0u64);
    for (vt, vv) in WE_EV {
        we_ev.v[vt.tousz()].v = vv;
    }

    //////////////////////////////////////////////
    // Solar Weight
    let mut we_so = PeaAssVar::from(0u64);
    for (vt, vv) in WE_RE {
        we_so.v[vt.tousz()].v = vv;
    }

    //////////////////////////////////////////////
    // ETruck Weight
    let mut we_et = PeaAssVar::from(0u64);
    for (vt, vv) in WE_ET {
        we_et.v[vt.tousz()].v = vv;
    }

    //////////////////////////////////////////////
    // EV bike Weight
    let mut we_eb = PeaAssVar::from(0u64);
    for (vt, vv) in WE_EB {
        we_eb.v[vt.tousz()].v = vv;
    }

    let mut we_tpa = PeaAssVar::from(0u64);
    for (vt, vv) in WE_TPA {
        we_tpa.v[vt.tousz()].v = vv;
    }

    let tpa_day_hours = arif.ass.v(TPA_DAY_HOURS);
    let tpa_price_thb = arif.ass.v(TPA_PRICE_THB);
    let tpa_ben_claim = arif.ass.v(TPA_BEN_CLAIM);
    let tpa_year_days = arif.ass.v(TPA_YEAR_DAYS);
    let tpafcs = arif.ass.ve(TPA_FORECAST)?;

    let NumValEnum::Json(tpa_json) = tpafcs else {
        return Err("No TPA forecast data".into());
    };
    let Value::Array(tpa_ary) = tpa_json else {
        return Err("TPA value is not array".into());
    };
    let mut tpa_fa = vec![];
    for va in tpa_ary {
        let Value::Number(n) = va else {
            continue;
        };
        let Some(f) = n.as_f64() else {
            continue;
        };
        tpa_fa.push(f as f32);
    }
    //println!("TPA FORECAST {tpa_fa:?}");

    //==== NORMALIZE1
    let mut tras_mx1 = PeaAssVar::from(0u64);
    for tras in assrw1.iter() {
        tras_mx1.max(tras);
    }
    let mut tras_raw = assrw1.clone();
    let mut tras_nor = tras_raw.clone();
    {
        let tik = std::time::SystemTime::now();
        thread::scope(|s| {
            for tras_nor in tras_nor.chunks_mut(sz) {
                let tras_mx1 = tras_mx1.clone();
                s.spawn(move || {
                    for tras in tras_nor {
                        tras.nor(&tras_mx1);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== start nor - {se} sec");
    }
    //==== EV CALCULATION
    {
        let tik = std::time::SystemTime::now();
        let mut tras_ev = tras_nor.clone();
        thread::scope(|s| {
            for (evs, rws) in tras_ev.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_ev = we_ev.clone();
                s.spawn(move || {
                    for (ev, rw) in evs.iter_mut().zip(rws.iter_mut()) {
                        ev.weigh(&we_ev);
                        ev.sum();
                        rw.v[VarType::HmChgEvTr as usize].v = ev.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== start ev - {se} sec");
    }

    //==== ET CALCULATION
    {
        let tik = std::time::SystemTime::now();
        let mut tras_et = tras_nor.clone();
        thread::scope(|s| {
            for (ets, rws) in tras_et.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_et = we_et.clone();
                s.spawn(move || {
                    for (et, rw) in ets.iter_mut().zip(rws.iter_mut()) {
                        et.weigh(&we_et);
                        et.sum();
                        rw.v[VarType::ChgEtTr as usize].v = et.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== start et - {se} sec");
    }

    //==== EB CALCULATION
    {
        let tik = std::time::SystemTime::now();
        let mut tras_eb = tras_nor.clone();
        thread::scope(|s| {
            for (ebs, rws) in tras_eb.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_eb = we_eb.clone();
                s.spawn(move || {
                    for (eb, rw) in ebs.iter_mut().zip(rws.iter_mut()) {
                        eb.weigh(&we_eb);
                        eb.sum();
                        rw.v[VarType::ChgEbTr as usize].v = eb.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== start eb - {se} secs");
    }

    //==== SOLAR ROOF
    {
        let tik = std::time::SystemTime::now();
        let mut tras_so = tras_nor.clone();
        thread::scope(|s| {
            for (sos, rws) in tras_so.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_so = we_so.clone();
                s.spawn(move || {
                    for (so, rw) in sos.iter_mut().zip(rws.iter_mut()) {
                        so.weigh(&we_so);
                        so.sum();
                        rw.v[VarType::SolarRoof as usize].v = so.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== start so - {se} secs");
    }

    //==== TRAN ASSESS SUMMARY
    let mut sum = PeaAssVar::from(0u64);
    {
        let tik = std::time::SystemTime::now();

        let (tx0, rx) = mpsc::channel();
        let mut txv = vec![];
        for _ in 1..coreno {
            txv.push(tx0.clone());
        }
        txv.push(tx0);
        thread::scope(|s| {
            for tras_raw in tras_raw.chunks_mut(sz) {
                let tx = txv.pop().unwrap();
                //let tx = tx0.clone();
                s.spawn(move || {
                    let mut sum = PeaAssVar::from(0u64);
                    for tras in tras_raw.iter() {
                        sum.add(tras);
                    }
                    tx.send(sum).unwrap();
                });
            }
        });
        for r in rx.iter() {
            sum.add(&r);
        }

        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== SUM - {se} secs");
    }

    //==== MAX2 CALCULATION
    let (tx0, rx) = mpsc::channel();
    let mut txv = vec![];
    for _ in 1..coreno {
        txv.push(tx0.clone());
    }
    txv.push(tx0);
    thread::scope(|s| {
        let tik = std::time::SystemTime::now();
        for tras_raw in tras_raw.chunks_mut(sz) {
            let tx = txv.pop().unwrap();
            s.spawn(move || {
                let mut max2 = PeaAssVar::from(0u64);
                for tr in tras_raw {
                    tr.v[VarType::LvPowSatTr as usize].v =
                        tr.v[VarType::PkPowTr as usize].v / z2o(tr.v[VarType::PwCapTr as usize].v);
                    tr.v[VarType::CntLvPowSatTr as usize].v =
                        if tr.v[VarType::LvPowSatTr as usize].v > 0.8f32 {
                            1f32
                        } else {
                            0f32
                        };
                    tr.v[VarType::ChgStnCap as usize].v = tr.v[VarType::ChgStnCapTr as usize].v;
                    tr.v[VarType::ChgStnSell as usize].v = tr.v[VarType::ChgStnSellTr as usize].v;
                    tr.v[VarType::MvPowSatTr as usize].v = tr.v[VarType::MaxPosPowSub as usize].v
                        / z2o(tr.v[VarType::SubPowCap as usize].v);
                    tr.v[VarType::MvVspp as usize].v = tr.v[VarType::VsppMv as usize].v;
                    tr.v[VarType::HvSpp as usize].v = tr.v[VarType::SppHv as usize].v;
                    tr.v[VarType::SmallSell as usize].v = tr.v[VarType::SmallSellTr as usize].v;
                    tr.v[VarType::LargeSell as usize].v = tr.v[VarType::LargeSellTr as usize].v;
                    tr.v[VarType::UnbalPow as usize].v = tr.v[VarType::UnbalPowTr as usize].v;
                    let v = tr.v[VarType::UnbalPowTr as usize].v
                        / z2o(tr.v[VarType::PwCapTr as usize].v);
                    tr.v[VarType::CntUnbalPow as usize].v = if v > 0.5f32 { 1f32 } else { 0f32 };

                    max2.max(tr);
                }
                let _ = tx.send(max2);
            });
        }
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== MAX2 - {se} secs");
    });
    let mut max2 = PeaAssVar::from(0u64);
    //let mut sum2 = PeaAssVar::from(0u64);
    for r in rx.iter() {
        //max2.add(&r);
        max2.max(&r);
    }
    // ----- checking

    //////////////////////////////////////
    let mut we_uc1 = PeaAssVar::from(0u64);
    for (vt, vv) in WE_UC1 {
        we_uc1.v[vt.tousz()].v = vv;
    }
    let mut we_uc2 = PeaAssVar::from(0u64);
    for (vt, vv) in WE_UC2 {
        we_uc2.v[vt.tousz()].v = vv;
    }
    let mut we_uc3 = PeaAssVar::from(0u64);
    for (vt, vv) in WE_UC3 {
        we_uc3.v[vt.tousz()].v = vv;
    }
    let evsc = ev_scurv();
    let resc = re_scurv();
    let etsc = et_scurv();
    let ebsc = eb_scurv();
    //println!("evsc: {} resc: {}", evsc.len(), resc.len());

    //==== NORMALIZE 2
    let mut tras_nor = tras_raw.clone();
    {
        let tik = std::time::SystemTime::now();
        thread::scope(|s| {
            for tras_nor in tras_nor.chunks_mut(sz) {
                let max2 = max2.clone();
                s.spawn(move || {
                    for tras in tras_nor {
                        tras.nor(&max2);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG3 ==== start nor - {se} secs");
    }

    //==== EV NORMALIZE
    {
        let ass = arif.assumption();
        let mut tras_sum = tras_raw.clone();
        let tik = std::time::SystemTime::now();
        let ev_bat_cap = ass.v(EV_BAT_CAP_MWH);
        let ev_chg_yr = ass.v(EV_TIME_FULCHG_YR);
        thread::scope(|s| {
            //let ac = ac.clone();
            for (trsum, trraw) in tras_sum.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let ass = ass.clone();
                let evsc = evsc.clone();
                let etsc = etsc.clone();
                let ebsc = ebsc.clone();
                let sum = sum.clone();
                s.spawn(move || {
                    //let ac = ac.clone();
                    for (tras, tras0) in trsum.iter_mut().zip(trraw.iter_mut()) {
                        tras.nor(&sum);

                        //============================== EV consumption
                        tras0.v[VarType::NoHmChgEvTr as usize].v =
                            tras.v[VarType::HmChgEvTr as usize].v * 210_000f32;
                        tras0.v[VarType::PowHmChgEvTr as usize].v =
                            tras0.v[VarType::NoHmChgEvTr as usize].v * 0.007f32;
                        for rt in evsc.iter() {
                            let evno =
                                tras.v[VarType::HmChgEvTr.tousz()].v * ass.v(EV_AT_2050) * rt;
                            tras0.vy[VarType::NoHmChgEvTr.tousz()].push(evno);
                            tras0.vy[VarType::PowHmChgEvTr.tousz()]
                                .push(evno * ass.v(EV_CHG_POW_KW) / 1_000f32);
                            let _evbt = evno
                                * ass.v(EV_CHG_POW_KW)
                                * ass.v(EV_DAY_CHG_HOUR)
                                * ass.v(EV_CHG_PROF_KW)
                                * 365.0
                                * ass.v(EV_CLAIM_RATE);
                            let evbt = evno * ev_bat_cap * ev_chg_yr 
                                * ass.v(EV_CLAIM_RATE);
                            tras0.vy[VarType::FirEvChgThb.tousz()].push(evbt);
                        }
                        //tras0.sum_yr(VarType::FirEvChgThb, &ass);
                        tras0.yr_sum(VarType::FirEvChgThb, &ass);

                        //============================== EV TRUCK consumption
                        // EV truck
                        for rt in etsc.iter() {
                            let etno = tras.v[VarType::ChgEtTr.tousz()].v * ass.v(ET_AT_2050) * rt;
                            tras0.vy[VarType::NoEtTr.tousz()].push(etno);
                            let etbt = etno
                                * ass.v(ET_CHG_POW_KW)
                                * ass.v(ET_DAY_CHG_HOUR)
                                * ass.v(EV_CHG_PROF_KW)
                                * 365.0
                                * ass.v(ET_CLAIM_RATE);
                            tras0.vy[VarType::FirEtChgThb.tousz()].push(etbt);
                        }
                        //tras0.sum_yr(VarType::FirEtChgThb, &ass);
                        tras0.yr_sum(VarType::FirEtChgThb, &ass);

                        // EV bike
                        for rt in ebsc.iter() {
                            let ebno = tras.v[VarType::ChgEbTr.tousz()].v * ass.v(ET_AT_2050) * rt;
                            tras0.vy[VarType::NoEtTr.tousz()].push(ebno);
                            let ebbt = ebno
                                * ass.v(EB_CHG_POW_KW)
                                * ass.v(EB_DAY_CHG_HOUR)
                                * ass.v(EV_CHG_PROF_KW)
                                * 365.0
                                * ass.v(EB_CLAIM_RATE);
                            tras0.vy[VarType::FirEbChgThb.tousz()].push(ebbt);
                        }
                        //tras0.sum_yr(VarType::FirEbChgThb, &ass);
                        tras0.yr_sum(VarType::FirEbChgThb, &ass);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG3 ==== CALC RATIO - {se} secs");
    }

    //==== USE CASE 1
    {
        let tik = std::time::SystemTime::now();
        let mut tras_uc1 = tras_nor.clone();
        thread::scope(|s| {
            for (truc1, trraw) in tras_uc1.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_uc1 = we_uc1.clone();
                s.spawn(move || {
                    for (tras, tras0) in truc1.iter_mut().zip(trraw.iter_mut()) {
                        tras.weigh(&we_uc1);
                        tras.sum();
                        tras0.v[VarType::Uc1Val as usize].v = tras.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG3 ==== USE CASE 1 - {se} secs");
    }

    //==== USE CASE 2
    {
        let tik = std::time::SystemTime::now();
        let mut tras_uc2 = tras_nor.clone();
        thread::scope(|s| {
            for (truc2, trraw) in tras_uc2.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_uc2 = we_uc2.clone();
                s.spawn(move || {
                    for (tras, tras0) in truc2.iter_mut().zip(trraw.iter_mut()) {
                        tras.weigh(&we_uc2);
                        tras.sum();
                        tras0.v[VarType::Uc2Val as usize].v = tras.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG3 ==== USE CASE 2 - {se} secs");
    }

    //==== USE CASE 3
    {
        let tik = std::time::SystemTime::now();
        let mut tras_uc3 = tras_nor.clone();
        thread::scope(|s| {
            for (truc3, trraw) in tras_uc3.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_uc3 = we_uc3.clone();
                s.spawn(move || {
                    for (tras, tras0) in truc3.iter_mut().zip(trraw.iter_mut()) {
                        tras.weigh(&we_uc3);
                        tras.sum();
                        tras0.v[VarType::Uc3Val as usize].v = tras.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG3 ==== USE CASE 3 - {se} secs");
    }

    //==== BENEFIT CALCULATION
    {
        let tik = std::time::SystemTime::now();
        thread::scope(|s| {
            for trraw in tras_raw.chunks_mut(sz) {
                let ass = ass.clone();
                s.spawn(move || {
                    for tras in trraw.iter_mut() {
                        let ary = crate::ben3::ben_bill_accu(tras, &ass);
                        tras.vy[VarType::FirBilAccu.tousz()].append(&mut ary.clone());
                        tras.yr_sum(VarType::FirBilAccu, &ass);

                        let csh = crate::ben3::ben_cash_flow(tras, &ass);
                        tras.vy[VarType::FirCashFlow.tousz()].append(&mut csh.clone());
                        tras.yr_sum(VarType::FirCashFlow, &ass);

                        let drs = crate::ben3::ben_dr_save(tras, &ass);
                        tras.vy[VarType::FirDRSave.tousz()].append(&mut drs.clone());
                        tras.yr_sum(VarType::FirDRSave, &ass);

                        let bxc = crate::ben3::ben_boxline_save(tras, &ass);
                        tras.vy[VarType::FirMetBoxSave.tousz()].append(&mut bxc.clone());
                        tras.yr_sum(VarType::FirMetBoxSave, &ass);

                        let wks = crate::ben3::ben_work_save(tras, &ass);
                        tras.vy[VarType::FirLaborSave.tousz()].append(&mut wks.clone());
                        tras.yr_sum(VarType::FirLaborSave, &ass);

                        let mts = crate::ben3::ben_sell_meter(tras, &ass);
                        tras.vy[VarType::FirMetSell.tousz()].append(&mut mts.clone());
                        tras.yr_sum(VarType::FirMetSell, &ass);

                        let ems = crate::ben3::ben_emeter(tras, &ass);
                        tras.vy[VarType::FirEMetSave.tousz()].append(&mut ems.clone());
                        tras.yr_sum(VarType::FirEMetSave, &ass);

                        let mrs = crate::ben3::ben_mt_read(tras, &ass);
                        tras.vy[VarType::FirMetReadSave.tousz()].append(&mut mrs.clone());
                        tras.yr_sum(VarType::FirMetReadSave, &ass);

                        let mds = crate::ben3::ben_mt_disconn(tras, &ass);
                        tras.vy[VarType::FirMetDisSave.tousz()].append(&mut mds.clone());
                        tras.yr_sum(VarType::FirMetDisSave, &ass);

                        let tos = crate::ben3::ben_tou_sell(tras, &ass);
                        tras.vy[VarType::FirTouSell.tousz()].append(&mut tos.clone());
                        tras.yr_sum(VarType::FirTouSell, &ass);

                        let trs = crate::ben3::ben_tou_read(tras, &ass);
                        tras.vy[VarType::FirTouReadSave.tousz()].append(&mut trs.clone());
                        tras.yr_sum(VarType::FirTouReadSave, &ass);

                        let tus = crate::ben3::ben_tou_update(tras, &ass);
                        tras.vy[VarType::FirTouUpdateSave.tousz()].append(&mut tus.clone());
                        tras.yr_sum(VarType::FirTouUpdateSave, &ass);

                        let ols = crate::ben3::ben_outage_labor(tras, &ass);
                        tras.vy[VarType::FirOutLabSave.tousz()].append(&mut ols.clone());
                        tras.yr_sum(VarType::FirOutLabSave, &ass);

                        let cps = crate::ben3::ben_reduce_complain(tras, &ass);
                        tras.vy[VarType::FirComplainSave.tousz()].append(&mut cps.clone());
                        tras.yr_sum(VarType::FirComplainSave, &ass);

                        let asv = crate::ben3::ben_asset_value(tras, &ass);
                        tras.vy[VarType::FirAssetValue.tousz()].append(&mut asv.clone());
                        tras.yr_sum(VarType::FirAssetValue, &ass);

                        let mes = crate::ben3::ben_model_entry(tras, &ass);
                        tras.vy[VarType::FirDataEntrySave.tousz()].append(&mut mes.clone());
                        tras.yr_sum(VarType::FirDataEntrySave, &ass);

                        let dum = vec![0f32; 15];
                        tras.vy[VarType::FirBatSubSave.tousz()].append(&mut dum.clone());
                        tras.yr_sum(VarType::FirBatSubSave, &ass);
                        tras.vy[VarType::FirBatSvgSave.tousz()].append(&mut dum.clone());
                        tras.yr_sum(VarType::FirBatSvgSave, &ass);
                        tras.vy[VarType::FirBatEnerSave.tousz()].append(&mut dum.clone());
                        tras.yr_sum(VarType::FirBatEnerSave, &ass);
                        tras.vy[VarType::FirBatPriceDiff.tousz()].append(&mut dum.clone());
                        tras.yr_sum(VarType::FirBatPriceDiff, &ass);

                        let nome1 = tras.v[VarType::NoMet1Ph.tousz()].v;
                        let nome3 = tras.v[VarType::NoMet3Ph.tousz()].v;
                        let notr = tras.v[VarType::NoPeaTr.tousz()].v;
                        let nobess = 0.0;
                        let bescap = 0.0;
                        let nodev = nome1 + nome3 + notr + nobess;

                        //let bescap = tras.v[VarType::PowHmChgEvTr.tousz()].v * BESS_EVPOW_MWH_MULT;
                        //tras.v[VarType::BessMWh.tousz()].v = bescap * bess_x;

                        tras.v[VarType::NoDevice.tousz()].v = nodev;

                        tras.vy[VarType::CstMet1pIns.tousz()].append(&mut cst_m1p_ins(nome1, &ass));
                        tras.yr_sum(VarType::CstMet1pIns, &ass);
                        tras.vy[VarType::CstMet3pIns.tousz()].append(&mut cst_m3p_ins(nome3, &ass));
                        tras.yr_sum(VarType::CstMet3pIns, &ass);
                        tras.vy[VarType::CstTrIns.tousz()].append(&mut cst_tr_ins(notr, &ass));
                        tras.yr_sum(VarType::CstTrIns, &ass);
                        tras.vy[VarType::CstBessIns.tousz()].append(&mut cst_bes_ins(bescap, &ass));
                        tras.yr_sum(VarType::CstBessIns, &ass);
                        tras.vy[VarType::CstPlfmIns.tousz()].append(&mut cst_plfm_ins(nodev, &ass));
                        tras.yr_sum(VarType::CstPlfmIns, &ass);
                        tras.vy[VarType::CstCommIns.tousz()].append(&mut cst_comm_ins(nodev, &ass));
                        tras.yr_sum(VarType::CstCommIns, &ass);

                        tras.vy[VarType::CstMet1pImp.tousz()].append(&mut cst_m1p_imp(nome1, &ass));
                        tras.yr_sum(VarType::CstMet1pImp, &ass);
                        tras.vy[VarType::CstMet3pImp.tousz()].append(&mut cst_m3p_imp(nome3, &ass));
                        tras.yr_sum(VarType::CstMet3pImp, &ass);
                        tras.vy[VarType::CstTrImp.tousz()].append(&mut cst_tr_imp(notr, &ass));
                        tras.yr_sum(VarType::CstTrImp, &ass);
                        tras.vy[VarType::CstBessImp.tousz()].append(&mut cst_bes_imp(bescap, &ass));
                        tras.yr_sum(VarType::CstBessImp, &ass);
                        tras.vy[VarType::CstPlfmImp.tousz()].append(&mut cst_plfm_imp(nodev, &ass));
                        tras.yr_sum(VarType::CstPlfmImp, &ass);
                        tras.vy[VarType::CstCommImp.tousz()].append(&mut cst_comm_imp(nodev, &ass));
                        tras.yr_sum(VarType::CstCommImp, &ass);

                        tras.vy[VarType::CstMet1pOp.tousz()].append(&mut cst_m1p_op(nome1, &ass));
                        tras.yr_sum(VarType::CstMet1pOp, &ass);
                        tras.vy[VarType::CstMet3pOp.tousz()].append(&mut cst_m3p_op(nome3, &ass));
                        tras.yr_sum(VarType::CstMet3pOp, &ass);
                        tras.vy[VarType::CstTrOp.tousz()].append(&mut cst_tr_op(notr, &ass));
                        tras.yr_sum(VarType::CstTrOp, &ass);
                        tras.vy[VarType::CstBessOp.tousz()].append(&mut cst_bes_op(bescap, &ass));
                        tras.yr_sum(VarType::CstBessOp, &ass);
                        tras.vy[VarType::CstPlfmOp.tousz()].append(&mut cst_plfm_op(nodev, &ass));
                        tras.yr_sum(VarType::CstPlfmOp, &ass);
                        tras.vy[VarType::CstCommOp.tousz()].append(&mut cst_comm_op(nodev, &ass));
                        tras.yr_sum(VarType::CstCommOp, &ass);

                        let sel = tras.v[VarType::AllSellTr.tousz()].v;

                        tras.vy[VarType::EirCustLossSave.tousz()]
                            .append(&mut eir_cust_loss_save(sel, &ass));
                        tras.yr_sum(VarType::EirCustLossSave, &ass);
                        tras.vy[VarType::EirConsumSave.tousz()]
                            .append(&mut eir_cust_save(sel, &ass));
                        tras.yr_sum(VarType::EirConsumSave, &ass);
                        tras.vy[VarType::EirGrnHsEmsSave.tousz()]
                            .append(&mut eir_ghg_save(sel, &ass));
                        tras.yr_sum(VarType::EirGrnHsEmsSave, &ass);
                        tras.vy[VarType::EirCustMvRev.tousz()]
                            .append(&mut eir_cust_mv_rev(sel, &ass));
                        tras.yr_sum(VarType::EirCustMvRev, &ass);
                        tras.vy[VarType::EirCustEvSave.tousz()]
                            .append(&mut eir_cust_ev_save(sel, &ass));
                        tras.yr_sum(VarType::EirCustEvSave, &ass);
                        tras.vy[VarType::EirCustEtrkSave.tousz()]
                            .append(&mut eir_cust_etruck_save(sel, &ass));
                        tras.yr_sum(VarType::EirCustEtrkSave, &ass);
                        tras.vy[VarType::EirSolaRfTopSave.tousz()]
                            .append(&mut eir_cust_solar_roof(sel, &ass));
                        tras.yr_sum(VarType::EirSolaRfTopSave, &ass);
                        tras.vy[VarType::EirEnerResvSave.tousz()]
                            .append(&mut eir_en_rev_save(sel, &ass));
                        tras.yr_sum(VarType::EirEnerResvSave, &ass);

                        ass_calc(tras, &ass).expect("?");
                        /*
                         */
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG4 ==== COST/BENEFIT - {se} secs");
    }
    for (i, rw) in tras_raw.iter_mut().enumerate() {
        rw.ix = i;
    }

    //====== TPO CALCULATION
    let mut unb_v: Vec<_> = tras_raw
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::FirUnbSave as usize].v, i))
        .collect();
    unb_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in unb_v.iter().enumerate() {
        let me1 = tras_raw[*i].v[VarType::NoMet1Ph.tousz()].v;
        let me3 = tras_raw[*i].v[VarType::NoMet3Ph.tousz()].v;
        //let me1 = 1.0;
        //let me3 = 1.0;
        if r < tpo_no {
            tras_raw[*i].v[VarType::TpoAdd as usize].v = 1.0;
            tras_raw[*i].v[VarType::NoMet1PhSim.tousz()].v = me1;
            tras_raw[*i].v[VarType::NoMet3PhSim.tousz()].v = me3;
            tras_raw[*i].v[VarType::NoPeaTrSim.tousz()].v = 1.0;
        } else if r < tpo_no + ecu_no {
            tras_raw[*i].v[VarType::EcuAdd as usize].v = 1.0;
            tras_raw[*i].v[VarType::NoMet1PhPlc.tousz()].v = me1;
            tras_raw[*i].v[VarType::NoMet3PhPlc.tousz()].v = me3;
            tras_raw[*i].v[VarType::NoPeaTrPlc.tousz()].v = 1.0;
        } else {
            tras_raw[*i].v[VarType::NoMet1PhSim.tousz()].v = me1;
            tras_raw[*i].v[VarType::NoMet3PhSim.tousz()].v = me3;
            tras_raw[*i].v[VarType::NoPeaTrSim.tousz()].v = 1.0;
        }
    }

    //==== BRANCH INITILIZE
    let mut m_brn = HashMap::<String, BranchGIS>::new();
    let mut m_brnas = HashMap::<String, PeaAssVar>::new();
    //let mut brn_m = HashMap::<String, PeaAssVar>::new();
    let aids = vec![
        "N1", "N2", "N3", "NE1", "NE2", "NE3", "C1", "C2", "C3", "S1", "S2", "S3",
    ];
    for aid in aids {
        let aojs0 = ProcEngine::aojs0(aid);
        for aoj in &aojs0 {
            let Some(ref aojcd) = aoj.code else {
                continue;
            };
            let brn = BranchGIS::from(aoj);
            m_brn.insert(aojcd.clone(), brn);
            let mut ass = PeaAssVar::from(0u64);
            ass.aoj = aojcd.to_string();
            m_brnas.insert(ass.aoj.clone(), ass);
        }
    }

    /*
        //==== SUMMARY TO FEEDER
        let a_feed_hm = Arc::new(Mutex::new(HashMap::<String, (PeaAssVar, Vec<usize>)>::new()));
        {
            let tik = std::time::SystemTime::now();
            std::thread::scope(|s| {
                let rsz = tras_raw.len().div_ceil(coreno);
                for vsa in tras_raw.chunks_mut(rsz) {
                    let c_feed_hm = a_feed_hm.clone();
                    s.spawn(move || {
                        let mut feed_m = HashMap::<String, (PeaAssVar, Vec<usize>)>::new();
                        for sa in vsa {
                            let sbid = sa.sbid.to_string();
                            let (sbas, sbis) = sub_m
                                .entry(sbid)
                                .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                            sbas.add_ex(sa, &SUB_LEVEL_FLDS);
                            sbas.pvid = sa.pvid.clone();
                            sbas.sbid = sa.sbid.clone();
                            sbis.push(sa.ix);
                        }
                        if let Ok(mut sub_hm) = c_sub_hm.lock() {
                            for (sub, mut sbis1) in sub_m.into_values() {
                                let sbid = sub.sbid.to_string();
                                let (sbas, sbis2) = sub_hm
                                    .entry(sbid)
                                    .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                                sbas.add_ex(&sub, &SUB_LEVEL_FLDS);
                                sbas.sbid = sub.sbid.clone();
                                sbas.pvid = sub.pvid.clone();
                                sbis2.append(&mut sbis1);
                            }
                            //println!("  sub chunk {}", sub_m.len());
                        }
                    });
                }
            });
            let se = tik.elapsed().unwrap().as_secs();
            println!("======== TR => SUBSTATION - {se} secs");
        }
        let sub_hm = a_sub_hm.lock().unwrap().clone();
        drop(a_sub_hm);
        let mut subass: Vec<_> = sub_hm.into_values().collect();

        //==== FEEDER CALCULATE
        {
            let tik = std::time::SystemTime::now();
            let ssz = subass.len().div_ceil(coreno);
            //========== CHECK
            std::thread::scope(|s| {
                for subasc in subass.chunks_mut(ssz) {
                    let resc = resc.clone();
                    let ac = ac.clone();
                    s.spawn(move || {
                        for (vas, _vis) in subasc.iter_mut() {
                            vas.v[VarType::NoMet1PhA.tousz()].v =
                                (vas.v[VarType::NoMet1Ph.tousz()].v * me_x).floor();
                            vas.v[VarType::NoMet3PhA.tousz()].v =
                                (vas.v[VarType::NoMet3Ph.tousz()].v * me_x).floor();

                            vas.v[VarType::NoMet1PhSimA.tousz()].v =
                                (vas.v[VarType::NoMet1PhSim.tousz()].v * me_x).floor();
                            vas.v[VarType::NoMet3PhSimA.tousz()].v =
                                (vas.v[VarType::NoMet3PhSim.tousz()].v * me_x).floor();
                            vas.v[VarType::NoMet1PhPlcA.tousz()].v =
                                (vas.v[VarType::NoMet1PhPlc.tousz()].v * me_x).floor();
                            vas.v[VarType::NoMet3PhPlcA.tousz()].v =
                                (vas.v[VarType::NoMet3PhPlc.tousz()].v * me_x).floor();
                            vas.v[VarType::NoMet1PhWisA.tousz()].v =
                                (vas.v[VarType::NoMet1PhWis.tousz()].v * me_x).floor();
                            vas.v[VarType::NoMet3PhWisA.tousz()].v =
                                (vas.v[VarType::NoMet3PhWis.tousz()].v * me_x).floor();

                            vas.v[VarType::ChgStnCap as usize].v =
                                vas.v[VarType::ChgStnCapTr as usize].v;
                            vas.v[VarType::ChgStnSell as usize].v =
                                vas.v[VarType::ChgStnSellTr as usize].v;
                            vas.v[VarType::MvPowSatTr as usize].v =
                                vas.v[VarType::MaxPosPowSub as usize].v
                                    / z2o(vas.v[VarType::SubPowCap as usize].v);
                            vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
                            vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
                            vas.v[VarType::SmallSell as usize].v =
                                vas.v[VarType::SmallSellTr as usize].v;
                            vas.v[VarType::LargeSell as usize].v =
                                vas.v[VarType::LargeSellTr as usize].v;
                            vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
                            let v = vas.v[VarType::UnbalPowTr as usize].v
                                / z2o(vas.v[VarType::PwCapTr as usize].v);
                            vas.v[VarType::CntUnbalPow as usize].v =
                                if v > 0.5f32 { 1f32 } else { 0f32 };

                            let pwmx = vas.v[VarType::MaxPosPowSub.tousz()].v;
                            for rt in resc.iter() {
                                let rerev = rt
                                    * pwmx
                                    * ac.v(PEAK_POWER_RATIO)
                                    * ac.v(RENEW_HOUR_PER_DAY)
                                    * 365.0
                                    * ac.v(RENEW_SAVE_PER_MWH);
                                vas.vy[VarType::FirMvReThb.tousz()].push(rerev);
                            }
                            vas.yr_sum(VarType::FirMvReThb, &ac);
                            // CHECK MEMBERS

                            let _ = ass_calc(vas, &ac);
                        }
                    });
                }
            });
            let se = tik.elapsed().unwrap().as_secs();
            println!("SUM SUB:{} - {se}sec", subass.len());
        }
    */

    //==== SUMMARY TO SUBST
    let a_sub_hm = Arc::new(Mutex::new(HashMap::<String, (PeaAssVar, Vec<usize>)>::new()));
    {
        let tik = std::time::SystemTime::now();
        std::thread::scope(|s| {
            let rsz = tras_raw.len().div_ceil(coreno);
            for vsa in tras_raw.chunks_mut(rsz) {
                let c_sub_hm = a_sub_hm.clone();
                s.spawn(move || {
                    let mut sub_m = HashMap::<String, (PeaAssVar, Vec<usize>)>::new();
                    for sa in vsa {
                        let sbid = sa.sbid.to_string();
                        let (sbas, sbis) = sub_m
                            .entry(sbid)
                            .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                        sbas.add_ex(sa, &SUB_LEVEL_FLDS);
                        sbas.pvid = sa.pvid.clone();
                        sbas.sbid = sa.sbid.clone();
                        sbis.push(sa.ix);
                    }
                    if let Ok(mut sub_hm) = c_sub_hm.lock() {
                        for (sub, mut sbis1) in sub_m.into_values() {
                            let sbid = sub.sbid.to_string();
                            let (sbas, sbis2) = sub_hm
                                .entry(sbid)
                                .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                            sbas.add_ex(&sub, &SUB_LEVEL_FLDS);
                            sbas.sbid = sub.sbid.clone();
                            sbas.pvid = sub.pvid.clone();
                            sbis2.append(&mut sbis1);
                        }
                        //println!("  sub chunk {}", sub_m.len());
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== TR => SUBSTATION - {se} secs");
    }
    let sub_hm = a_sub_hm.lock().unwrap().clone();
    drop(a_sub_hm);
    let mut subass: Vec<_> = sub_hm.into_values().collect();

    //==== SUBSTATION CALCULATE
    {
        let ass = ass.clone();
        let tik = std::time::SystemTime::now();
        let ssz = subass.len().div_ceil(coreno);
        //========== CHECK
        std::thread::scope(|s| {
            let ass = ass.clone();
            for subasc in subass.chunks_mut(ssz) {
                let ass = ass.clone();
                let resc = resc.clone();
                //let ac = ac.clone();
                s.spawn(move || {
                    for (vas, _vis) in subasc.iter_mut() {
                        vas.v[VarType::NoMet1PhA.tousz()].v =
                            (vas.v[VarType::NoMet1Ph.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhA.tousz()].v =
                            (vas.v[VarType::NoMet3Ph.tousz()].v * me_x).floor();

                        vas.v[VarType::NoMet1PhSimA.tousz()].v =
                            (vas.v[VarType::NoMet1PhSim.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhSimA.tousz()].v =
                            (vas.v[VarType::NoMet3PhSim.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet1PhPlcA.tousz()].v =
                            (vas.v[VarType::NoMet1PhPlc.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhPlcA.tousz()].v =
                            (vas.v[VarType::NoMet3PhPlc.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet1PhWisA.tousz()].v =
                            (vas.v[VarType::NoMet1PhWis.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhWisA.tousz()].v =
                            (vas.v[VarType::NoMet3PhWis.tousz()].v * me_x).floor();

                        vas.v[VarType::ChgStnCap as usize].v =
                            vas.v[VarType::ChgStnCapTr as usize].v;
                        vas.v[VarType::ChgStnSell as usize].v =
                            vas.v[VarType::ChgStnSellTr as usize].v;
                        vas.v[VarType::MvPowSatTr as usize].v =
                            vas.v[VarType::MaxPosPowSub as usize].v
                                / z2o(vas.v[VarType::SubPowCap as usize].v);
                        vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
                        vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
                        vas.v[VarType::SmallSell as usize].v =
                            vas.v[VarType::SmallSellTr as usize].v;
                        vas.v[VarType::LargeSell as usize].v =
                            vas.v[VarType::LargeSellTr as usize].v;
                        vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
                        let v = vas.v[VarType::UnbalPowTr as usize].v
                            / z2o(vas.v[VarType::PwCapTr as usize].v);
                        vas.v[VarType::CntUnbalPow as usize].v =
                            if v > 0.5f32 { 1f32 } else { 0f32 };

                        let pwmx = vas.v[VarType::MaxPosPowSub.tousz()].v;
                        for rt in resc.iter() {
                            let rerev = rt
                                * pwmx
                                * ass.v(PEAK_POWER_RATIO)
                                * ass.v(RENEW_HOUR_PER_DAY)
                                * 365.0
                                * ass.v(RENEW_SAVE_PER_MWH);
                            vas.vy[VarType::FirMvReThb.tousz()].push(rerev);
                        }
                        vas.yr_sum(VarType::FirMvReThb, &ass);
                        // CHECK MEMBERS

                        let _ = ass_calc(vas, &ass);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("SUM SUB:{} - {se}sec", subass.len());
    }

    //==== CREATE SUBST MEMBER FILE
    {
        let tik = std::time::SystemTime::now();
        for (subass, trasis) in subass.iter() {
            let sid = subass.sbid.clone();
            let mut sbrw = Vec::<PeaAssVar>::new();
            for ti in trasis.iter() {
                let rat = tras_raw[*ti].v[VarType::AllSellTr.tousz()].v
                    / z2o(subass.v[VarType::AllSellTr.tousz()].v);
                tras_raw[*ti].v[VarType::MaxPosPowSub.tousz()].v =
                    subass.v[VarType::MaxPosPowSub.tousz()].v * rat;
                tras_raw[*ti].v[VarType::FirMvReThb.tousz()].v =
                    subass.v[VarType::FirMvReThb.tousz()].v * rat;

                tras_raw[*ti].vy[VarType::MaxPosPowSub.tousz()] = subass.vy
                    [VarType::MaxPosPowSub.tousz()]
                .iter()
                .map(|v| *v * rat)
                .collect::<Vec<_>>();

                tras_raw[*ti].vy[VarType::FirMvReThb.tousz()] = subass.vy
                    [VarType::FirMvReThb.tousz()]
                .iter()
                .map(|v| *v * rat)
                .collect::<Vec<_>>();

                sbrw.push(tras_raw[*ti].clone());
            }
            let fnm = format!("{dnm}/{sid}-rw4.bin");
            //println!("save {sid} - v:{} -> {fnm}", trasis.len());
            let bin: Vec<u8> = bincode::encode_to_vec(&sbrw, bincode::config::standard()).unwrap();
            std::fs::write(fnm, bin).unwrap();
        }
        let se = tik.elapsed().unwrap().as_secs();
        println!("SAVE FILES:{} - {se}sec", subass.len());
    }
    /*
    for (sid, ixv) in sub_mi {
        let mut sbid = String::new();
        for ix in &ixv {
            sbid = tras_raw[*ix].sbid.clone();
        }
    }
    */

    //==== SUMMARY TO BRANCH
    let a_brn_hm = Arc::new(Mutex::new(HashMap::<String, PeaAssVar>::new()));
    let a_brn_mi = Arc::new(Mutex::new(HashMap::<String, Vec<usize>>::new()));
    {
        let tik = std::time::SystemTime::now();
        std::thread::scope(|s| {
            let rsz = tras_raw.len().div_ceil(coreno);
            for vsa in tras_raw.chunks_mut(rsz) {
                let c_brn_hm = a_brn_hm.clone();
                let c_brn_mi = a_brn_mi.clone();
                s.spawn(move || {
                    let mut brn_m = HashMap::<String, PeaAssVar>::new();
                    let mut brn_i = HashMap::<String, Vec<usize>>::new();
                    for sa in vsa {
                        let aojcd = sa.aojcd.clone();
                        let bras = brn_m.entry(aojcd).or_insert_with(|| PeaAssVar::from(0u64));
                        bras.add_ex(sa, &SUB_LEVEL_FLDS);
                        bras.pvid = sa.pvid.clone();
                        bras.aojcd = sa.aojcd.to_string();

                        let aojcd = sa.aojcd.clone();
                        let bris = brn_i.entry(aojcd).or_default();
                        bris.push(sa.ix);
                    }
                    if let Ok(mut brn_hm) = c_brn_hm.lock() {
                        for brn in brn_m.values() {
                            let brid = brn.aojcd.clone();
                            let bras = brn_hm.entry(brid).or_insert_with(|| brn.clone());
                            bras.add_ex(brn, &SUB_LEVEL_FLDS);
                        }
                    }
                    if let Ok(mut brn_mi) = c_brn_mi.lock() {
                        for (brid, mut ixv2) in brn_i {
                            let brid = brid.clone();
                            let ixv1 = brn_mi.entry(brid).or_default();
                            ixv1.append(&mut ixv2);
                        }
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== TR => BRANCH - {se} secs");
    }
    let brn_hm = a_brn_hm.lock().unwrap().clone();
    drop(a_brn_hm);
    let mut brnass: Vec<_> = brn_hm.into_values().collect();

    //==== BRANCH CALCULATE
    {
        let ass = ass.clone();
        let tik = std::time::SystemTime::now();
        std::thread::scope(|s| {
            let ass = ass.clone();
            let bsz = brnass.len().div_ceil(coreno);
            for brnasc in brnass.chunks_mut(bsz) {
                let ass = ass.clone();
                s.spawn(move || {
                    for vas in brnasc.iter_mut() {
                        vas.v[VarType::ChgStnCap as usize].v =
                            vas.v[VarType::ChgStnCapTr as usize].v;
                        vas.v[VarType::ChgStnSell as usize].v =
                            vas.v[VarType::ChgStnSellTr as usize].v;
                        vas.v[VarType::MvPowSatTr as usize].v =
                            vas.v[VarType::MaxPosPowSub as usize].v
                                / z2o(vas.v[VarType::SubPowCap as usize].v);
                        vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
                        vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
                        vas.v[VarType::SmallSell as usize].v =
                            vas.v[VarType::SmallSellTr as usize].v;
                        vas.v[VarType::LargeSell as usize].v =
                            vas.v[VarType::LargeSellTr as usize].v;
                        vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
                        let v = vas.v[VarType::UnbalPowTr as usize].v
                            / z2o(vas.v[VarType::PwCapTr as usize].v);
                        vas.v[VarType::CntUnbalPow as usize].v =
                            if v > 0.5f32 { 1f32 } else { 0f32 };
                        let _ = ass_calc(vas, &ass);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("SUM SUB:{} BRN:{} - {se}sec", subass.len(), brnass.len());
    }

    println!("====================================");
    println!("====================================");
    println!("=========== ASSUM LEN: {}", subass.len());
    let sola: f32 = subass
        .iter()
        .map(|s| s.0.v[VarType::SolarEnergy as usize].v)
        .sum();
    println!("  =========== SubSolarEnergy: {sola}");

    //let bess_solar = 0.50;
    //let maxbess = 20.0;

    let bess_solar = arif.ass.v(SOLAR_TO_BESS_FACTOR);
    let maxbess = arif.ass.v(SUB_BESS_MAX_MWH);
    println!("============  bess: {bess_solar}  maxbess:{maxbess}");

    let sb_svg: Vec<String> = if let Ok(prvs) = arif.ass.ve(SVG_PILOT_SUBST) {
        println!("========= SVG SUBSTATION =======");
        let mut sbsvg = vec![];
        if let NumValEnum::Json(Value::Array(prvs)) = prvs {
            for x in prvs.iter() {
                let Value::String(x) = x else {
                    continue;
                };
                let s = x.to_string();
                println!(" {s}");
                sbsvg.push(s);
            }
        }
        sbsvg
    } else {
        vec![]
    };

    //tras.v[VarType::BessMWh.tousz()].v = bescap * bess_x;
    //===== SVG Calculation
    let mut svg_v: Vec<_> = subass
        .iter()
        .enumerate()
        .map(|(i, s)| (s.0.v[VarType::SolarEnergy as usize].v, i))
        .collect();
    svg_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (_r, (_, i)) in svg_v.iter().enumerate() {
        if sb_svg.contains(&subass[*i].0.sbid) {
            println!(">>>>>>>>>>>>>>> SVG add to {}", subass[*i].0.sbid);
            subass[*i].0.v[VarType::SvgAdd as usize].v = 1.0;
        } else if subass[*i].0.v[VarType::SolarEnergy as usize].v > 3.0 {
            let bess = subass[*i].0.v[VarType::SolarEnergy as usize].v * bess_solar;
            let bess = if bess > maxbess { maxbess } else { bess };
            subass[*i].0.v[VarType::BessMWh as usize].v = bess;
        }
        /*
        if r < svg_no {
            //subass[*i].0.v[VarType::SvgAdd as usize].v = 1.0;
            subass[*i].0.v[VarType::SolarEnergy as usize].v = 1.0;
        }
        */
    }

    //==== PROVINCE CALCULATION
    let a_prv_hm = Arc::new(Mutex::new(HashMap::<String, PeaAssVar>::new()));
    let tik = std::time::SystemTime::now();
    {
        std::thread::scope(|s| {
            let psz = subass.len().div_ceil(coreno);
            for vsa in subass.chunks_mut(psz) {
                let c_prv_hm = a_prv_hm.clone();
                s.spawn(move || {
                    let mut prv_m = HashMap::<String, PeaAssVar>::new();
                    for (sa, _) in vsa {
                        let pvid = sa.pvid.to_string();
                        //let pvas = prv_m.entry(pvid).or_insert_with(|| sa.clone());
                        //pvas.add_ex(sa, &PRV_LEVEL_FLDS);
                        if let Some(pvas) = prv_m.get_mut(&pvid) {
                            pvas.add_ex(sa, &PRV_LEVEL_FLDS);
                        } else {
                            prv_m.insert(pvid, sa.clone());
                        }
                    }
                    if let Ok(mut prv_hm) = c_prv_hm.lock() {
                        for prv in prv_m.values() {
                            let pvid = prv.pvid.to_string();
                            //let pvas = prv_hm.entry(pvid).or_insert_with(|| prv.clone());
                            //pvas.add_ex(prv, &SUB_LEVEL_FLDS);
                            if let Some(pvas) = prv_hm.get_mut(&pvid) {
                                pvas.add_ex(prv, &SUB_LEVEL_FLDS);
                            } else {
                                prv_hm.insert(pvid, prv.clone());
                            }
                        }
                        //println!("  prv chunk {}", prv_m.len());
                    }
                });
            }
        });
    }
    let prv_hm = a_prv_hm.lock().unwrap().clone();
    drop(a_prv_hm);
    let mut prvass: Vec<_> = prv_hm.into_values().collect();
    for vas in prvass.iter_mut() {
        vas.v[VarType::ChgStnCap as usize].v = vas.v[VarType::ChgStnCapTr as usize].v;
        vas.v[VarType::ChgStnSell as usize].v = vas.v[VarType::ChgStnSellTr as usize].v;
        vas.v[VarType::MvPowSatTr as usize].v =
            vas.v[VarType::MaxPosPowSub as usize].v / z2o(vas.v[VarType::SubPowCap as usize].v);
        vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
        vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
        vas.v[VarType::SmallSell as usize].v = vas.v[VarType::SmallSellTr as usize].v;
        vas.v[VarType::LargeSell as usize].v = vas.v[VarType::LargeSellTr as usize].v;
        vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
        vas.v[VarType::TpaZone.tousz()].v /= z2o(vas.v[VarType::NoTr.tousz()].v);
        //let v = vas.v[VarType::UnbalPowTr as usize].v / z2o(vas.v[VarType::PwCapTr as usize].v);
        //vas.v[VarType::CntUnbalPow as usize].v = if v > 0.5f32 { 1f32 } else { 0f32 };
        let _ = ass_calc(vas, &ass);
    }
    let mut pvas_mx = PeaAssVar::from(0u64);
    for pvas in prvass.iter() {
        pvas_mx.max(pvas);
    }
    let mut pvas_no = prvass.clone();
    for pvas in pvas_no.iter_mut() {
        pvas.nor(&pvas_mx);
    }
    for (pvas, pvno) in prvass.iter_mut().zip(pvas_no.iter()) {
        pvas.v[VarType::TpaZone.tousz()].v = pvno.v[VarType::TpaZone.tousz()].v;
        pvas.v[VarType::MaxPosPowSub.tousz()].v = pvno.v[VarType::MaxPosPowSub.tousz()].v;
    }
    let mut tpa_ad = PeaAssVar::from(0u64);
    let mut pvtpa = prvass.clone();
    let _flds = [VarType::TpaFcst.tousz()];
    for (tpa, prv) in pvtpa.iter_mut().zip(prvass.iter_mut()) {
        tpa.weigh(&we_tpa);
        tpa.sum();
        prv.v[VarType::TpaFcst as usize].v = tpa.res;
        tpa_ad.v[VarType::TpaFcst.tousz()].v += prv.v[VarType::TpaFcst.tousz()].v;
        //tpa_ad.add_ex(prv, &flds);
    }

    let mut pvas_no = prvass.clone();
    for (no, prv) in pvas_no.iter_mut().zip(prvass.iter_mut()) {
        no.nor(&tpa_ad);
        prv.v[VarType::TpaFcst.tousz()].v = no.v[VarType::TpaFcst.tousz()].v;
    }

    for pvas in prvass.iter_mut() {
        let rat = pvas.v[VarType::TpaFcst.tousz()].v;
        let mut tpav = vec![];
        for y in tpa_fa.iter().take(15) {
            let y = y * tpa_day_hours * tpa_price_thb * tpa_year_days;
            let y = y * rat;
            let y = y * tpa_ben_claim;
            let y = y * 1_000f32;
            tpav.push(y);
        }
        pvas.vy[VarType::FirTpaThb.tousz()].append(&mut tpav);
        pvas.yr_sum(VarType::FirTpaThb, &ass);
    }

    let se = tik.elapsed().unwrap().as_secs();
    println!("PRV:{} - {se}sec", prvass.len());

    //==== SAVE SUMMARY FILE
    {
        let mut subass0 = subass
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<PeaAssVar>>();

        let tik = std::time::SystemTime::now();
        ass_rank(&mut subass0);
        let bin: Vec<u8> = bincode::encode_to_vec(&subass0, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-sbrw.bin"), bin).unwrap();

        ass_rank(&mut prvass);
        let bin: Vec<u8> = bincode::encode_to_vec(&prvass, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-pvrw.bin"), bin).unwrap();

        ass_rank(&mut brnass);
        let bin: Vec<u8> = bincode::encode_to_vec(&brnass, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-aojrw.bin"), bin).unwrap();
        let se = tik.elapsed().unwrap().as_secs();
        println!("SAVE summary - {se} sec",);
    }

    Ok(())
}
*/

pub fn stage_02_d(
    coreno: usize,
    arif: &ArchiInfo,
    mut tras_raw: Vec<PeaAssVar>,
) -> Result<(), Box<dyn Error>> {
    let ass = arif.assumption();
    let dnm = ass.t(OUTDIR);
    let me_x = arif.ass.v(METER_NO_MULTIPLY);
    //let bess_x = arif.ass.v(BESS_EVCAP_MULTIPLY);
    //let evsc = ev_scurv();
    /*
    let resc = re_scurv();
    let yrst = ass.u(PRJ_START_YEAR);
    let yred = ass.u(PRJ_END_YEAR);
    */
    let resc = ev_scurv2(ass.u(PRJ_START_YEAR), ass.u(PRJ_END_YEAR));
    //let etsc = et_scurv();
    //let ebsc = eb_scurv();

    let mut we_tpa = PeaAssVar::from(0u64);
    for (vt, vv) in WE_TPA {
        we_tpa.v[vt.tousz()].v = vv;
    }

    let tpa_day_hours = arif.ass.v(TPA_DAY_HOURS);
    let tpa_price_thb = arif.ass.v(TPA_PRICE_THB);
    let tpa_ben_claim = arif.ass.v(TPA_BEN_CLAIM);
    let tpa_year_days = arif.ass.v(TPA_YEAR_DAYS);
    let tpafcs = ass.ve(TPA_FORECAST)?;
    let NumValEnum::Json(tpa_json) = tpafcs else {
        return Err("No TPA forecast data".into());
    };
    let Value::Array(tpa_ary) = tpa_json else {
        return Err("TPA value is not array".into());
    };
    let mut tpa_fa = vec![];
    for va in tpa_ary {
        let Value::Number(n) = va else {
            continue;
        };
        let Some(f) = n.as_f64() else {
            continue;
        };
        tpa_fa.push(f as f32);
    }
    //==== BRANCH INITILIZE
    let mut m_brn = HashMap::<String, BranchGIS>::new();
    let mut m_brnas = HashMap::<String, PeaAssVar>::new();
    //let mut brn_m = HashMap::<String, PeaAssVar>::new();
    let aids = vec![
        "N1", "N2", "N3", "NE1", "NE2", "NE3", "C1", "C2", "C3", "S1", "S2", "S3",
    ];
    for aid in aids {
        let aojs0 = ProcEngine::aojs0(aid);
        for aoj in &aojs0 {
            let Some(ref aojcd) = aoj.code else {
                continue;
            };
            let brn = BranchGIS::from(aoj);
            m_brn.insert(aojcd.clone(), brn);
            let mut ass = PeaAssVar::from(0u64);
            ass.aoj = aojcd.to_string();
            m_brnas.insert(ass.aoj.clone(), ass);
        }
    }

    //==== SUMMARY TO FEEDER

    //============  SUMMARY TO SUBST BEGIN ===========
    //============  SUMMARY TO SUBST BEGIN ===========
    //============  SUMMARY TO SUBST BEGIN ===========
    let a_sub_hm = Arc::new(Mutex::new(HashMap::<String, (PeaAssVar, Vec<usize>)>::new()));
    {
        let tik = std::time::SystemTime::now();
        std::thread::scope(|s| {
            let rsz = tras_raw.len().div_ceil(coreno);
            for vsa in tras_raw.chunks_mut(rsz) {
                let c_sub_hm = a_sub_hm.clone();
                s.spawn(move || {
                    let mut sub_m = HashMap::<String, (PeaAssVar, Vec<usize>)>::new();
                    for sa in vsa {
                        let sbid = sa.sbid.to_string();
                        let (sbas, sbis) = sub_m
                            .entry(sbid)
                            .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                        sbas.add_ex(sa, &SUB_LEVEL_FLDS);
                        sbas.pvid = sa.pvid.clone();
                        sbas.sbid = sa.sbid.clone();
                        sbis.push(sa.ix);
                    }
                    if let Ok(mut sub_hm) = c_sub_hm.lock() {
                        for (sub, mut sbis1) in sub_m.into_values() {
                            let sbid = sub.sbid.to_string();
                            let (sbas, sbis2) = sub_hm
                                .entry(sbid)
                                .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                            sbas.add_ex(&sub, &SUB_LEVEL_FLDS);
                            sbas.sbid = sub.sbid.clone();
                            sbas.pvid = sub.pvid.clone();
                            sbis2.append(&mut sbis1);
                        }
                        //println!("  sub chunk {}", sub_m.len());
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== TR => SUBSTATION - {se} secs");
    }
    let sub_hm = a_sub_hm.lock().unwrap().clone();
    drop(a_sub_hm);
    let mut subass: Vec<_> = sub_hm.into_values().collect();

    //==== SUBSTATION CALCULATE
    {
        let ass = ass.clone();
        let tik = std::time::SystemTime::now();
        let ssz = subass.len().div_ceil(coreno);

        //========== CHECK
        std::thread::scope(|s| {
            let ass = ass.clone();
            for subasc in subass.chunks_mut(ssz) {
                let ass = ass.clone();
                let resc = resc.clone();
                //let ac = ac.clone();
                s.spawn(move || {
                    for (vas, _vis) in subasc.iter_mut() {
                        vas.v[VarType::NoMet1PhA.tousz()].v =
                            (vas.v[VarType::NoMet1Ph.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhA.tousz()].v =
                            (vas.v[VarType::NoMet3Ph.tousz()].v * me_x).floor();

                        vas.v[VarType::NoMet1PhSimA.tousz()].v =
                            (vas.v[VarType::NoMet1PhSim.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhSimA.tousz()].v =
                            (vas.v[VarType::NoMet3PhSim.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet1PhPlcA.tousz()].v =
                            (vas.v[VarType::NoMet1PhPlc.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhPlcA.tousz()].v =
                            (vas.v[VarType::NoMet3PhPlc.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet1PhWisA.tousz()].v =
                            (vas.v[VarType::NoMet1PhWis.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhWisA.tousz()].v =
                            (vas.v[VarType::NoMet3PhWis.tousz()].v * me_x).floor();

                        vas.v[VarType::ChgStnCap as usize].v =
                            vas.v[VarType::ChgStnCapTr as usize].v;
                        vas.v[VarType::ChgStnSell as usize].v =
                            vas.v[VarType::ChgStnSellTr as usize].v;
                        vas.v[VarType::MvPowSatTr as usize].v =
                            vas.v[VarType::MaxPosPowSub as usize].v
                                / z2o(vas.v[VarType::SubPowCap as usize].v);
                        vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
                        vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
                        vas.v[VarType::SmallSell as usize].v =
                            vas.v[VarType::SmallSellTr as usize].v;
                        vas.v[VarType::LargeSell as usize].v =
                            vas.v[VarType::LargeSellTr as usize].v;
                        vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
                        let v = vas.v[VarType::UnbalPowTr as usize].v
                            / z2o(vas.v[VarType::PwCapTr as usize].v);
                        vas.v[VarType::CntUnbalPow as usize].v =
                            if v > 0.5f32 { 1f32 } else { 0f32 };

                        let pwmx = vas.v[VarType::MaxPosPowSub.tousz()].v;
                        for rt in resc.iter() {
                            let rerev = rt
                                * pwmx
                                * ass.v(PEAK_POWER_RATIO)
                                * ass.v(RENEW_HOUR_PER_DAY)
                                * 365.0
                                * ass.v(RENEW_SAVE_PER_MWH);
                            vas.vy[VarType::FirMvReThb.tousz()].push(rerev);
                        }
                        vas.yr_sum(VarType::FirMvReThb, &ass);
                        // CHECK MEMBERS

                        let _ = ass_calc(vas, &ass);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("SUM SUB:{} - {se}sec", subass.len());
    }
    {
        let tik = std::time::SystemTime::now();
        for (subass, trasis) in subass.iter() {
            let sid = subass.sbid.clone();
            let mut sbrw = Vec::<PeaAssVar>::new();
            for ti in trasis.iter() {
                let rat = tras_raw[*ti].v[VarType::AllSellTr.tousz()].v
                    / z2o(subass.v[VarType::AllSellTr.tousz()].v);
                tras_raw[*ti].v[VarType::MaxPosPowSub.tousz()].v =
                    subass.v[VarType::MaxPosPowSub.tousz()].v * rat;
                tras_raw[*ti].v[VarType::FirMvReThb.tousz()].v =
                    subass.v[VarType::FirMvReThb.tousz()].v * rat;

                tras_raw[*ti].vy[VarType::MaxPosPowSub.tousz()] = subass.vy
                    [VarType::MaxPosPowSub.tousz()]
                .iter()
                .map(|v| *v * rat)
                .collect::<Vec<_>>();

                tras_raw[*ti].vy[VarType::FirMvReThb.tousz()] = subass.vy
                    [VarType::FirMvReThb.tousz()]
                .iter()
                .map(|v| *v * rat)
                .collect::<Vec<_>>();

                sbrw.push(tras_raw[*ti].clone());
            }
            let fnm = format!("{dnm}/{sid}-rw4.bin");
            //println!("save {sid} - v:{} -> {fnm}", trasis.len());
            let bin: Vec<u8> = bincode::encode_to_vec(&sbrw, bincode::config::standard()).unwrap();
            std::fs::write(fnm, bin).unwrap();
        }
        let se = tik.elapsed().unwrap().as_secs();
        println!("SAVE FILES:{} - {se}sec", subass.len());
    }
    //============  SUMMARY TO SUBST END ===========
    //============  SUMMARY TO SUBST END ===========
    //============  SUMMARY TO SUBST END ===========



    let (brns, cd_bri) = get_brn_map()?;
    //============  SUMMARY TO BRANCH BEGIN ===========
    //============  SUMMARY TO BRANCH BEGIN ===========
    //============  SUMMARY TO BRANCH BEGIN ===========
    let a_branch_hm = Arc::new(Mutex::new(HashMap::<String, (PeaAssVar, Vec<usize>)>::new()));
    {
        let tik = std::time::SystemTime::now();
        std::thread::scope(|s| {
            //let brns = brns.clone();
            //let cd_bri = cd_bri.clone();
            let rsz = tras_raw.len().div_ceil(coreno);
            for vsa in tras_raw.chunks_mut(rsz) {
                let brns = brns.clone();
                let cd_bri = cd_bri.clone();
                let c_branch_hm = a_branch_hm.clone();
                s.spawn(move || {
                    let mut brn_m = HashMap::<String, (PeaAssVar, Vec<usize>)>::new();
                    for sa in vsa {
                        let aojcd = sa.aojcd.to_string();
                        let (bras, bris) = brn_m
                            .entry(aojcd.clone())
                            .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                        bras.add_ex(sa, &SUB_LEVEL_FLDS);
                        bras.aojcd = aojcd.clone();
                        if let Some(ii) = cd_bri.get(&aojcd) && let Some(ji) = &brns[*ii].i_prov {
                            bras.pvid = brns[*ji].name.clone();
                        }
                        bris.push(sa.ix);
                    }
                    if let Ok(mut brn_hm) = c_branch_hm.lock() {
                        for (brn, mut brnis) in brn_m.into_values() {
                            let aojcd = brn.aojcd.clone();
                            let (bras, bris) = brn_hm
                                .entry(aojcd.clone())
                                .or_insert_with(|| (PeaAssVar::from(0u64), Vec::<usize>::new()));
                            bras.add_ex(&brn, &SUB_LEVEL_FLDS);
                            bras.aojcd = brn.aojcd.clone();
                            if let Some(ii) = cd_bri.get(&aojcd) && let Some(ji) = &brns[*ii].i_prov {
                                bras.pvid = brns[*ji].name.clone();
                            }
                            let mut bris = bris.clone();
                            brnis.append(&mut bris);
                        }
                        //println!("  sub chunk {}", sub_m.len());
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== TR => BRANCH - {se} secs");
    }
    let branch_hm = a_branch_hm.lock().unwrap().clone();
    drop(a_branch_hm);
    let mut branch_ass: Vec<_> = branch_hm.into_values().collect();

    //==== SUBSTATION CALCULATE
    {
        let ass = ass.clone();
        let tik = std::time::SystemTime::now();
        let ssz = branch_ass.len().div_ceil(coreno);

        //========== CHECK
        std::thread::scope(|s| {
            let ass = ass.clone();
            for subasc in branch_ass.chunks_mut(ssz) {
                let ass = ass.clone();
                let resc = resc.clone();
                //let ac = ac.clone();
                s.spawn(move || {
                    for (vas, _vis) in subasc.iter_mut() {
                        vas.v[VarType::NoMet1PhA.tousz()].v =
                            (vas.v[VarType::NoMet1Ph.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhA.tousz()].v =
                            (vas.v[VarType::NoMet3Ph.tousz()].v * me_x).floor();

                        vas.v[VarType::NoMet1PhSimA.tousz()].v =
                            (vas.v[VarType::NoMet1PhSim.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhSimA.tousz()].v =
                            (vas.v[VarType::NoMet3PhSim.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet1PhPlcA.tousz()].v =
                            (vas.v[VarType::NoMet1PhPlc.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhPlcA.tousz()].v =
                            (vas.v[VarType::NoMet3PhPlc.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet1PhWisA.tousz()].v =
                            (vas.v[VarType::NoMet1PhWis.tousz()].v * me_x).floor();
                        vas.v[VarType::NoMet3PhWisA.tousz()].v =
                            (vas.v[VarType::NoMet3PhWis.tousz()].v * me_x).floor();

                        vas.v[VarType::ChgStnCap as usize].v =
                            vas.v[VarType::ChgStnCapTr as usize].v;
                        vas.v[VarType::ChgStnSell as usize].v =
                            vas.v[VarType::ChgStnSellTr as usize].v;
                        vas.v[VarType::MvPowSatTr as usize].v =
                            vas.v[VarType::MaxPosPowSub as usize].v
                                / z2o(vas.v[VarType::SubPowCap as usize].v);
                        vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
                        vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
                        vas.v[VarType::SmallSell as usize].v =
                            vas.v[VarType::SmallSellTr as usize].v;
                        vas.v[VarType::LargeSell as usize].v =
                            vas.v[VarType::LargeSellTr as usize].v;
                        vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
                        let v = vas.v[VarType::UnbalPowTr as usize].v
                            / z2o(vas.v[VarType::PwCapTr as usize].v);
                        vas.v[VarType::CntUnbalPow as usize].v =
                            if v > 0.5f32 { 1f32 } else { 0f32 };

                        let pwmx = vas.v[VarType::MaxPosPowSub.tousz()].v;
                        for rt in resc.iter() {
                            let rerev = rt
                                * pwmx
                                * ass.v(PEAK_POWER_RATIO)
                                * ass.v(RENEW_HOUR_PER_DAY)
                                * 365.0
                                * ass.v(RENEW_SAVE_PER_MWH);
                            vas.vy[VarType::FirMvReThb.tousz()].push(rerev);
                        }
                        vas.yr_sum(VarType::FirMvReThb, &ass);
                        // CHECK MEMBERS

                        let _ = ass_calc(vas, &ass);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("SUM SUB:{} - {se}sec", branch_ass.len());
    }
    {
        let tik = std::time::SystemTime::now();
        for (subass, trasis) in branch_ass.iter() {
            let aojcd = subass.aojcd.clone();
            let mut sbrw = Vec::<PeaAssVar>::new();
            for ti in trasis.iter() {
                let rat = tras_raw[*ti].v[VarType::AllSellTr.tousz()].v
                    / z2o(subass.v[VarType::AllSellTr.tousz()].v);
                tras_raw[*ti].v[VarType::MaxPosPowSub.tousz()].v =
                    subass.v[VarType::MaxPosPowSub.tousz()].v * rat;
                tras_raw[*ti].v[VarType::FirMvReThb.tousz()].v =
                    subass.v[VarType::FirMvReThb.tousz()].v * rat;

                tras_raw[*ti].vy[VarType::MaxPosPowSub.tousz()] = subass.vy
                    [VarType::MaxPosPowSub.tousz()]
                .iter()
                .map(|v| *v * rat)
                .collect::<Vec<_>>();

                tras_raw[*ti].vy[VarType::FirMvReThb.tousz()] = subass.vy
                    [VarType::FirMvReThb.tousz()]
                .iter()
                .map(|v| *v * rat)
                .collect::<Vec<_>>();

                sbrw.push(tras_raw[*ti].clone());
            }
            let fnm = format!("{dnm}/{aojcd}-rw4.bin");
            //println!("save {sid} - v:{} -> {fnm}", trasis.len());
            let bin: Vec<u8> = bincode::encode_to_vec(&sbrw, bincode::config::standard()).unwrap();
            std::fs::write(fnm, bin).unwrap();
        }
        let se = tik.elapsed().unwrap().as_secs();
        println!("SAVE FILES:{} - {se}sec", branch_ass.len());
    }
    //============  SUMMARY TO BRANCH END ===========
    //============  SUMMARY TO BRANCH END ===========
    //============  SUMMARY TO BRANCH END ===========

    //============  SUMMARY TO BRANCH0 BEGIN ===========
    //============  SUMMARY TO BRANCH0 BEGIN ===========
    //============  SUMMARY TO BRANCH0 BEGIN ===========
    //==== SUMMARY TO BRANCH
    let a_brn_hm = Arc::new(Mutex::new(HashMap::<String, PeaAssVar>::new()));
    let a_brn_mi = Arc::new(Mutex::new(HashMap::<String, Vec<usize>>::new()));
    {
        let mut tc = 0;
        let mut m1 = 0;
        for rw in tras_raw.iter() {
            tc += rw.v[VarType::NoPeaTr as usize].v as i32;
            m1 += rw.v[VarType::NoMet1Ph as usize].v as i32;
        }
        println!("============= tc:{tc} m1:{m1}");
        let tik = std::time::SystemTime::now();
        std::thread::scope(|s| {
            let rsz = tras_raw.len().div_ceil(coreno);
            for vsa in tras_raw.chunks_mut(rsz) {
                let c_brn_hm = a_brn_hm.clone();
                let c_brn_mi = a_brn_mi.clone();
                s.spawn(move || {
                    let mut brn_m = HashMap::<String, PeaAssVar>::new();
                    let mut brn_i = HashMap::<String, Vec<usize>>::new();
                    for sa in vsa {
                        let aojcd = sa.aojcd.clone();
                        let bras = brn_m.entry(aojcd).or_insert_with(|| PeaAssVar::from(0u64));
                        bras.add_ex(sa, &SUB_LEVEL_FLDS);
                        bras.pvid = sa.pvid.clone();
                        bras.aojcd = sa.aojcd.to_string();

                        let aojcd = sa.aojcd.clone();
                        let bris = brn_i.entry(aojcd).or_default();
                        bris.push(sa.ix);
                    }
                    if let Ok(mut brn_hm) = c_brn_hm.lock() {
                        for brn in brn_m.values() {
                            let brid = brn.aojcd.clone();
                            //let bras = brn_hm.entry(brid).or_insert_with(|| brn.clone());
                            let bras = brn_hm.entry(brid).or_insert_with(|| PeaAssVar::from(0u64));
                            bras.add_ex(brn, &SUB_LEVEL_FLDS);
                            bras.aojcd = brn.aojcd.clone();
                        }
                    }
                    if let Ok(mut brn_mi) = c_brn_mi.lock() {
                        for (brid, mut ixv2) in brn_i {
                            let brid = brid.clone();
                            let ixv1 = brn_mi.entry(brid).or_default();
                            ixv1.append(&mut ixv2);
                        }
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== TR => BRANCH - {se} secs");
    }
    let brn_hm = a_brn_hm.lock().unwrap().clone();
    drop(a_brn_hm);
    let mut brnass: Vec<_> = brn_hm.into_values().collect();

    //==== BRANCH CALCULATE
    {
        let ass = ass.clone();
        let tik = std::time::SystemTime::now();
        std::thread::scope(|s| {
            let ass = ass.clone();
            let bsz = brnass.len().div_ceil(coreno);
            for brnasc in brnass.chunks_mut(bsz) {
                let ass = ass.clone();
                s.spawn(move || {
                    for vas in brnasc.iter_mut() {
                        vas.v[VarType::ChgStnCap as usize].v =
                            vas.v[VarType::ChgStnCapTr as usize].v;
                        vas.v[VarType::ChgStnSell as usize].v =
                            vas.v[VarType::ChgStnSellTr as usize].v;
                        vas.v[VarType::MvPowSatTr as usize].v =
                            vas.v[VarType::MaxPosPowSub as usize].v
                                / z2o(vas.v[VarType::SubPowCap as usize].v);
                        vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
                        vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
                        vas.v[VarType::SmallSell as usize].v =
                            vas.v[VarType::SmallSellTr as usize].v;
                        vas.v[VarType::LargeSell as usize].v =
                            vas.v[VarType::LargeSellTr as usize].v;
                        vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
                        let v = vas.v[VarType::UnbalPowTr as usize].v
                            / z2o(vas.v[VarType::PwCapTr as usize].v);
                        vas.v[VarType::CntUnbalPow as usize].v =
                            if v > 0.5f32 { 1f32 } else { 0f32 };
                        let _ = ass_calc(vas, &ass);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("SUM SUB:{} BRN:{} - {se}sec", subass.len(), brnass.len());
    }
    //============  SUMMARY TO BRANCH0 END ===========
    //============  SUMMARY TO BRANCH0 END ===========
    //============  SUMMARY TO BRANCH0 END ===========

    println!("====================================");
    println!("====================================");
    println!("=========== ASSUM LEN: {}", subass.len());
    let sola: f32 = subass
        .iter()
        .map(|s| s.0.v[VarType::SolarEnergy as usize].v)
        .sum();
    println!("  =========== SubSolarEnergy: {sola}");

    //let bess_solar = 0.50;
    //let maxbess = 20.0;

    let bess_solar = arif.ass.v(SOLAR_TO_BESS_FACTOR);
    let maxbess = arif.ass.v(SUB_BESS_MAX_MWH);
    println!("============  bess: {bess_solar}  maxbess:{maxbess}");

    /*
    let sb_svg: Vec<String> = if let Ok(prvs) = arif.ass.ve(SVG_PILOT_SUBST) {
        println!("========= SVG SUBSTATION =======");
        let mut sbsvg = vec![];
        if let NumValEnum::Json(Value::Array(prvs)) = prvs {
            for x in prvs.iter() {
                let Value::String(x) = x else {
                    continue;
                };
                let s = x.to_string();
                println!(" {s}");
                sbsvg.push(s);
            }
        }
        sbsvg
    } else {
        vec![]
    };
    */

    //tras.v[VarType::BessMWh.tousz()].v = bescap * bess_x;
    //===== SVG Calculation
    /*
    let mut svg_v: Vec<_> = subass
        .iter()
        .enumerate()
        .map(|(i, s)| (s.0.v[VarType::SolarEnergy as usize].v, i))
        .collect();
    svg_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (_r, (_, i)) in svg_v.iter().enumerate() {
        if sb_svg.contains(&subass[*i].0.sbid) {
            println!(">>>>>>>>>>>>>>> SVG add to {}", subass[*i].0.sbid);
            subass[*i].0.v[VarType::SvgAdd as usize].v = 1.0;
        } else if subass[*i].0.v[VarType::SolarEnergy as usize].v > 3.0 {
            let bess = subass[*i].0.v[VarType::SolarEnergy as usize].v * bess_solar;
            let bess = if bess > maxbess { maxbess } else { bess };
            subass[*i].0.v[VarType::BessMWh as usize].v = bess;
        }
    }
    */

    //============= PROVINCE CALCULATION BEGIN ===============
    let a_prv_hm = Arc::new(Mutex::new(HashMap::<String, PeaAssVar>::new()));
    let tik = std::time::SystemTime::now();
    {
        std::thread::scope(|s| {
            let psz = subass.len().div_ceil(coreno);
            for vsa in subass.chunks_mut(psz) {
                let c_prv_hm = a_prv_hm.clone();
                s.spawn(move || {
                    let mut prv_m = HashMap::<String, PeaAssVar>::new();
                    for (sa, _) in vsa {
                        let pvid = sa.pvid.to_string();
                        //let pvas = prv_m.entry(pvid).or_insert_with(|| sa.clone());
                        //pvas.add_ex(sa, &PRV_LEVEL_FLDS);
                        if let Some(pvas) = prv_m.get_mut(&pvid) {
                            pvas.add_ex(sa, &PRV_LEVEL_FLDS);
                        } else {
                            prv_m.insert(pvid, sa.clone());
                        }
                    }
                    if let Ok(mut prv_hm) = c_prv_hm.lock() {
                        for prv in prv_m.values() {
                            let pvid = prv.pvid.to_string();
                            //let pvas = prv_hm.entry(pvid).or_insert_with(|| prv.clone());
                            //pvas.add_ex(prv, &SUB_LEVEL_FLDS);
                            if let Some(pvas) = prv_hm.get_mut(&pvid) {
                                pvas.add_ex(prv, &SUB_LEVEL_FLDS);
                            } else {
                                prv_hm.insert(pvid, prv.clone());
                            }
                        }
                        //println!("  prv chunk {}", prv_m.len());
                    }
                });
            }
        });
    }
    let prv_hm = a_prv_hm.lock().unwrap().clone();
    drop(a_prv_hm);
    let mut prvass: Vec<_> = prv_hm.into_values().collect();
    for vas in prvass.iter_mut() {
        vas.v[VarType::ChgStnCap as usize].v = vas.v[VarType::ChgStnCapTr as usize].v;
        vas.v[VarType::ChgStnSell as usize].v = vas.v[VarType::ChgStnSellTr as usize].v;
        vas.v[VarType::MvPowSatTr as usize].v =
            vas.v[VarType::MaxPosPowSub as usize].v / z2o(vas.v[VarType::SubPowCap as usize].v);
        vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
        vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
        vas.v[VarType::SmallSell as usize].v = vas.v[VarType::SmallSellTr as usize].v;
        vas.v[VarType::LargeSell as usize].v = vas.v[VarType::LargeSellTr as usize].v;
        vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
        vas.v[VarType::TpaZone.tousz()].v /= z2o(vas.v[VarType::NoTr.tousz()].v);
        //let v = vas.v[VarType::UnbalPowTr as usize].v / z2o(vas.v[VarType::PwCapTr as usize].v);
        //vas.v[VarType::CntUnbalPow as usize].v = if v > 0.5f32 { 1f32 } else { 0f32 };
        let _ = ass_calc(vas, &ass);
    }
    let mut pvas_mx = PeaAssVar::from(0u64);
    for pvas in prvass.iter() {
        pvas_mx.max(pvas);
    }
    let mut pvas_no = prvass.clone();
    for pvas in pvas_no.iter_mut() {
        pvas.nor(&pvas_mx);
    }
    for (pvas, pvno) in prvass.iter_mut().zip(pvas_no.iter()) {
        pvas.v[VarType::TpaZone.tousz()].v = pvno.v[VarType::TpaZone.tousz()].v;
        pvas.v[VarType::MaxPosPowSub.tousz()].v = pvno.v[VarType::MaxPosPowSub.tousz()].v;
    }
    let mut tpa_ad = PeaAssVar::from(0u64);
    let mut pvtpa = prvass.clone();
    let _flds = [VarType::TpaFcst.tousz()];
    for (tpa, prv) in pvtpa.iter_mut().zip(prvass.iter_mut()) {
        tpa.weigh(&we_tpa);
        tpa.sum();
        prv.v[VarType::TpaFcst as usize].v = tpa.res;
        tpa_ad.v[VarType::TpaFcst.tousz()].v += prv.v[VarType::TpaFcst.tousz()].v;
        //tpa_ad.add_ex(prv, &flds);
    }

    let mut pvas_no = prvass.clone();
    for (no, prv) in pvas_no.iter_mut().zip(prvass.iter_mut()) {
        no.nor(&tpa_ad);
        prv.v[VarType::TpaFcst.tousz()].v = no.v[VarType::TpaFcst.tousz()].v;
    }

    for pvas in prvass.iter_mut() {
        let rat = pvas.v[VarType::TpaFcst.tousz()].v;
        let mut tpav = vec![];
        for y in tpa_fa.iter().take(15) {
            let y = y * tpa_day_hours * tpa_price_thb * tpa_year_days;
            let y = y * rat;
            let y = y * tpa_ben_claim;
            let y = y * 1_000f32;
            tpav.push(y);
        }
        pvas.vy[VarType::FirTpaThb.tousz()].append(&mut tpav);
        pvas.yr_sum(VarType::FirTpaThb, &ass);
    }

    let se = tik.elapsed().unwrap().as_secs();
    println!("PRV:{} - {se}sec", prvass.len());
    //============= PROVINCE CALCULATION END ===============

    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    //============= PROVINCE ## BRANCH CALCULATION BEGIN ===============
    let a_prv2_hm = Arc::new(Mutex::new(HashMap::<String, PeaAssVar>::new()));
    let tik = std::time::SystemTime::now();
    {
        std::thread::scope(|s| {
            let psz = branch_ass.len().div_ceil(coreno);
            for vsa in branch_ass.chunks_mut(psz) {
                let c_prv2_hm = a_prv2_hm.clone();
                s.spawn(move || {
                    let mut prv2_m = HashMap::<String, PeaAssVar>::new();
                    for (sa, _) in vsa {
                        let pvid = sa.pvid.to_string();
                        //let pvas = prv_m.entry(pvid).or_insert_with(|| sa.clone());
                        //pvas.add_ex(sa, &PRV_LEVEL_FLDS);
                        if let Some(pvas) = prv2_m.get_mut(&pvid) {
                            pvas.add_ex(sa, &PRV_LEVEL_FLDS);
                        } else {
                            prv2_m.insert(pvid, sa.clone());
                        }
                    }
                    if let Ok(mut prv2_hm) = c_prv2_hm.lock() {
                        for prv in prv2_m.values() {
                            let pvid = prv.pvid.to_string();
                            //let pvas = prv_hm.entry(pvid).or_insert_with(|| prv.clone());
                            //pvas.add_ex(prv, &SUB_LEVEL_FLDS);
                            if let Some(pvas) = prv2_hm.get_mut(&pvid) {
                                pvas.add_ex(prv, &SUB_LEVEL_FLDS);
                            } else {
                                prv2_hm.insert(pvid, prv.clone());
                            }
                        }
                        //println!("  prv chunk {}", prv_m.len());
                    }
                });
            }
        });
    }
    let prv2_hm = a_prv2_hm.lock().unwrap().clone();
    drop(a_prv2_hm);
    let mut prv2ass: Vec<_> = prv2_hm.into_values().collect();
    for vas in prv2ass.iter_mut() {
        vas.v[VarType::ChgStnCap as usize].v = vas.v[VarType::ChgStnCapTr as usize].v;
        vas.v[VarType::ChgStnSell as usize].v = vas.v[VarType::ChgStnSellTr as usize].v;
        vas.v[VarType::MvPowSatTr as usize].v =
            vas.v[VarType::MaxPosPowSub as usize].v / z2o(vas.v[VarType::SubPowCap as usize].v);
        vas.v[VarType::MvVspp as usize].v = vas.v[VarType::VsppMv as usize].v;
        vas.v[VarType::HvSpp as usize].v = vas.v[VarType::SppHv as usize].v;
        vas.v[VarType::SmallSell as usize].v = vas.v[VarType::SmallSellTr as usize].v;
        vas.v[VarType::LargeSell as usize].v = vas.v[VarType::LargeSellTr as usize].v;
        vas.v[VarType::UnbalPow as usize].v = vas.v[VarType::UnbalPowTr as usize].v;
        vas.v[VarType::TpaZone.tousz()].v /= z2o(vas.v[VarType::NoTr.tousz()].v);
        //let v = vas.v[VarType::UnbalPowTr as usize].v / z2o(vas.v[VarType::PwCapTr as usize].v);
        //vas.v[VarType::CntUnbalPow as usize].v = if v > 0.5f32 { 1f32 } else { 0f32 };
        let _ = ass_calc(vas, &ass);
    }
    let mut pv2as_mx = PeaAssVar::from(0u64);
    for pvas in prv2ass.iter() {
        pv2as_mx.max(pvas);
    }
    let mut pv2as_no = prv2ass.clone();
    for pvas in pv2as_no.iter_mut() {
        pvas.nor(&pv2as_mx);
    }
    for (pvas, pvno) in prv2ass.iter_mut().zip(pv2as_no.iter()) {
        pvas.v[VarType::TpaZone.tousz()].v = pvno.v[VarType::TpaZone.tousz()].v;
        pvas.v[VarType::MaxPosPowSub.tousz()].v = pvno.v[VarType::MaxPosPowSub.tousz()].v;
    }
    let mut tpa_ad = PeaAssVar::from(0u64);
    let mut pvtpa = prvass.clone();
    let _flds = [VarType::TpaFcst.tousz()];
    for (tpa, prv) in pvtpa.iter_mut().zip(prvass.iter_mut()) {
        tpa.weigh(&we_tpa);
        tpa.sum();
        prv.v[VarType::TpaFcst as usize].v = tpa.res;
        tpa_ad.v[VarType::TpaFcst.tousz()].v += prv.v[VarType::TpaFcst.tousz()].v;
        //tpa_ad.add_ex(prv, &flds);
    }

    let mut pv2as_no = prv2ass.clone();
    for (no, prv) in pv2as_no.iter_mut().zip(prv2ass.iter_mut()) {
        no.nor(&tpa_ad);
        prv.v[VarType::TpaFcst.tousz()].v = no.v[VarType::TpaFcst.tousz()].v;
    }

    for pvas in prv2ass.iter_mut() {
        let rat = pvas.v[VarType::TpaFcst.tousz()].v;
        let mut tpav = vec![];
        for y in tpa_fa.iter().take(15) {
            let y = y * tpa_day_hours * tpa_price_thb * tpa_year_days;
            let y = y * rat;
            let y = y * tpa_ben_claim;
            let y = y * 1_000f32;
            tpav.push(y);
        }
        pvas.vy[VarType::FirTpaThb.tousz()].append(&mut tpav);
        pvas.yr_sum(VarType::FirTpaThb, &ass);
    }

    let se = tik.elapsed().unwrap().as_secs();
    println!("PRV:{} - {se}sec", prv2ass.len());
    //============= PROVINCE ## BRANCH CALCULATION END ===============
    //============= PROVINCE ## BRANCH CALCULATION END ===============
    //============= PROVINCE ## BRANCH CALCULATION END ===============

    //==== SAVE SUMMARY FILE
    {
        let mut subass0 = subass
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<PeaAssVar>>();

        let tik = std::time::SystemTime::now();
        ass_rank(&mut subass0);
        let bin: Vec<u8> = bincode::encode_to_vec(&subass0, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-sbrw.bin"), bin).unwrap();

        ass_rank(&mut prvass);
        let bin: Vec<u8> = bincode::encode_to_vec(&prvass, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-pvrw.bin"), bin).unwrap();

        ass_rank(&mut brnass);
        let bin: Vec<u8> = bincode::encode_to_vec(&brnass, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-aojrw.bin"), bin).unwrap();

        let mut branch_ass = branch_ass
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<PeaAssVar>>();
        ass_rank(&mut branch_ass);
        let bin: Vec<u8> =
            bincode::encode_to_vec(&branch_ass, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-branch.bin"), bin).unwrap();

        ass_rank(&mut prv2ass);
        let bin: Vec<u8> = bincode::encode_to_vec(&prv2ass, bincode::config::standard()).unwrap();
        std::fs::write(format!("{dnm}/000-pvbrn.bin"), bin).unwrap();

        let se = tik.elapsed().unwrap().as_secs();
        println!("SAVE summary - {se} sec",);
    }

    Ok(())
}

pub const YR_SCURV_BEG: usize = 2021;
pub const YR_SCURV_END: usize = 2050;

pub fn ev_scurv2(styr: usize, edyr: usize) -> Vec<f32> {
    let mut curv = Vec::<f32>::new();
    for y in styr..=edyr {
        let aa = 1.410f32;
        let y0 = - 12.21f32;
        let b = (y - YR_SCURV_BEG) as f32 + y0;
        let c = b * 0.41f32;
        let d = c + 1.205f32;
        let d = -d;
        let e = d.exp();
        let f = 1f32 / (1f32 + e);
        let g = f.powf(aa);
        curv.push(g);
    }
    curv
}

pub fn et_scurv2(styr: usize, edyr: usize) -> Vec<f32> {
    let mut curv = Vec::<f32>::new();
    for y in styr..=edyr {
        let aa = 1.410f32;
        let y0 = - 12.21f32;
        let b = (y - YR_SCURV_BEG) as f32 + y0;
        let c = b * 0.41f32;
        let d = c + 1.205f32;
        let d = -d;
        let e = d.exp();
        let f = 1f32 / (1f32 + e);
        let g = f.powf(aa);
        curv.push(g);
    }
    curv
}

pub fn eb_scurv2(styr: usize, edyr: usize) -> Vec<f32> {
    let mut curv = Vec::<f32>::new();
    for y in styr..=edyr {
        let aa = 1.410f32;
        let y0 = - 12.21f32;
        let b = (y - YR_SCURV_BEG) as f32 + y0;
        let c = b * 0.41f32;
        let d = c + 1.205f32;
        let d = -d;
        let e = d.exp();
        let f = 1f32 / (1f32 + e);
        let g = f.powf(aa);
        curv.push(g);
    }
    curv
}

pub fn re_scurv2(styr: usize, edyr: usize) -> Vec<f32> {
    let mut curv = Vec::<f32>::new();
    for y in styr..=edyr {
        let a = (y - YR_SCURV_BEG) as f32;
        let b = a - 14f32;
        //let c = b * 0.3f32;
        let c = b * 0.41f32;
        //let d = c + 0.0f32;
        let d = c + 1.205f32;
        let d = -d;
        let e = d.exp();
        let f = 1f32 / (1f32 + e);
        //let g = f.powf(1f32);
        let g = f.powf(1.1f32);
        curv.push(g);
    }
    curv
}

pub fn stage_02_c(
    coreno: usize,
    arif: &ArchiInfo,
    assrw1: Vec<PeaAssVar>,
) -> Result<Vec<PeaAssVar>, Box<dyn Error>> {
    //  Assumption Constants
    let ass = arif.assumption();
    let me_x = arif.ass.v(METER_NO_MULTIPLY);
    let bess_x = arif.ass.v(BESS_EVCAP_MULTIPLY);

    println!("============ me_x: {me_x} bess_x : {bess_x}");

    //let cn = 10;
    //let sz = (assrw1.len() + cn - 1) / cn;
    let sz = assrw1.len().div_ceil(coreno);
    //let dnm = arif.ass.t(OUTDIR);

    /*
    let tpo_no = 20_000;
    let ecu_no = 10_000;
    let svg_no = 300;
     */

    let tpo_no = arif.ass.v(TPO_DEV_NO_REQ);
    let ecu_no = arif.ass.v(ECU_DEV_NO_REQ);
    //println!("===== TPO: {tpo_no}");
    //println!("===== ECU: {ecu_no}");

    let tpo_no = tpo_no as usize;
    let ecu_no = ecu_no as usize;
    /*
    let svg_no = arif.ass.v(SVG_DEV_NO_REQ);
    let svg_no = svg_no as usize;
    println!("===== SVG: {svg_no}");
    */

    //////////////////////////////////////////////
    // EV Weight
    let mut we_ev = PeaAssVar::from(0u64);
    for (vt, vv) in WE_EV {
        we_ev.v[vt.tousz()].v = vv;
    }

    //////////////////////////////////////////////
    // Solar Weight
    let mut we_so = PeaAssVar::from(0u64);
    for (vt, vv) in WE_RE {
        we_so.v[vt.tousz()].v = vv;
    }

    //////////////////////////////////////////////
    // ETruck Weight
    let mut we_et = PeaAssVar::from(0u64);
    for (vt, vv) in WE_ET {
        we_et.v[vt.tousz()].v = vv;
    }

    //////////////////////////////////////////////
    // EV bike Weight
    let mut we_eb = PeaAssVar::from(0u64);
    for (vt, vv) in WE_EB {
        we_eb.v[vt.tousz()].v = vv;
    }

    let mut we_tpa = PeaAssVar::from(0u64);
    for (vt, vv) in WE_TPA {
        we_tpa.v[vt.tousz()].v = vv;
    }

    //let tpa_day_hours = arif.ass.v(TPA_DAY_HOURS);
    //let tpa_price_thb = arif.ass.v(TPA_PRICE_THB);
    //let tpa_ben_claim = arif.ass.v(TPA_BEN_CLAIM);
    //let tpa_year_days = arif.ass.v(TPA_YEAR_DAYS);
    let tpafcs = arif.ass.ve(TPA_FORECAST)?;

    let NumValEnum::Json(tpa_json) = tpafcs else {
        return Err("No TPA forecast data".into());
    };
    let Value::Array(tpa_ary) = tpa_json else {
        return Err("TPA value is not array".into());
    };
    let mut tpa_fa = vec![];
    for va in tpa_ary {
        let Value::Number(n) = va else {
            continue;
        };
        let Some(f) = n.as_f64() else {
            continue;
        };
        tpa_fa.push(f as f32);
    }
    //println!("TPA FORECAST {tpa_fa:?}");

    //==== NORMALIZE1
    let mut tras_mx1 = PeaAssVar::from(0u64);
    for tras in assrw1.iter() {
        tras_mx1.max(tras);
    }
    let mut tras_raw = assrw1.clone();
    let mut tras_nor = tras_raw.clone();
    {
        let tik = std::time::SystemTime::now();
        thread::scope(|s| {
            for tras_nor in tras_nor.chunks_mut(sz) {
                let tras_mx1 = tras_mx1.clone();
                s.spawn(move || {
                    for tras in tras_nor {
                        tras.nor(&tras_mx1);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== start nor - {se} sec");
    }
    //==== EV CALCULATION
    {
        let tik = std::time::SystemTime::now();
        let mut tras_ev = tras_nor.clone();
        thread::scope(|s| {
            for (evs, rws) in tras_ev.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_ev = we_ev.clone();
                s.spawn(move || {
                    for (ev, rw) in evs.iter_mut().zip(rws.iter_mut()) {
                        ev.weigh(&we_ev);
                        ev.sum();
                        rw.v[VarType::HmChgEvTr as usize].v = ev.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== start ev - {se} sec");
    }

    //==== ET CALCULATION
    {
        let tik = std::time::SystemTime::now();
        let mut tras_et = tras_nor.clone();
        thread::scope(|s| {
            for (ets, rws) in tras_et.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_et = we_et.clone();
                s.spawn(move || {
                    for (et, rw) in ets.iter_mut().zip(rws.iter_mut()) {
                        et.weigh(&we_et);
                        et.sum();
                        rw.v[VarType::ChgEtTr as usize].v = et.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== start et - {se} sec");
    }

    //==== EB CALCULATION
    {
        let tik = std::time::SystemTime::now();
        let mut tras_eb = tras_nor.clone();
        thread::scope(|s| {
            for (ebs, rws) in tras_eb.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_eb = we_eb.clone();
                s.spawn(move || {
                    for (eb, rw) in ebs.iter_mut().zip(rws.iter_mut()) {
                        eb.weigh(&we_eb);
                        eb.sum();
                        rw.v[VarType::ChgEbTr as usize].v = eb.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== start eb - {se} secs");
    }

    //==== SOLAR ROOF
    {
        let tik = std::time::SystemTime::now();
        let mut tras_so = tras_nor.clone();
        thread::scope(|s| {
            for (sos, rws) in tras_so.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_so = we_so.clone();
                s.spawn(move || {
                    for (so, rw) in sos.iter_mut().zip(rws.iter_mut()) {
                        so.weigh(&we_so);
                        so.sum();
                        rw.v[VarType::SolarRoof as usize].v = so.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== start so - {se} secs");
    }

    //==== TRAN ASSESS SUMMARY
    let mut sum = PeaAssVar::from(0u64);
    {
        let tik = std::time::SystemTime::now();

        let (tx0, rx) = mpsc::channel();
        let mut txv = vec![];
        for _ in 1..coreno {
            txv.push(tx0.clone());
        }
        txv.push(tx0);
        thread::scope(|s| {
            for tras_raw in tras_raw.chunks_mut(sz) {
                let tx = txv.pop().unwrap();
                //let tx = tx0.clone();
                s.spawn(move || {
                    let mut sum = PeaAssVar::from(0u64);
                    for tras in tras_raw.iter() {
                        sum.add(tras);
                    }
                    tx.send(sum).unwrap();
                });
            }
        });
        for r in rx.iter() {
            sum.add(&r);
        }

        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== SUM - {se} secs");
    }

    //==== MAX2 CALCULATION
    let (tx0, rx) = mpsc::channel();
    let mut txv = vec![];
    for _ in 1..coreno {
        txv.push(tx0.clone());
    }
    txv.push(tx0);
    thread::scope(|s| {
        let tik = std::time::SystemTime::now();
        for tras_raw in tras_raw.chunks_mut(sz) {
            let tx = txv.pop().unwrap();
            s.spawn(move || {
                let mut max2 = PeaAssVar::from(0u64);
                for tr in tras_raw {
                    tr.v[VarType::LvPowSatTr as usize].v =
                        tr.v[VarType::PkPowTr as usize].v / z2o(tr.v[VarType::PwCapTr as usize].v);
                    tr.v[VarType::CntLvPowSatTr as usize].v =
                        if tr.v[VarType::LvPowSatTr as usize].v > 0.8f32 {
                            1f32
                        } else {
                            0f32
                        };
                    tr.v[VarType::ChgStnCap as usize].v = tr.v[VarType::ChgStnCapTr as usize].v;
                    tr.v[VarType::ChgStnSell as usize].v = tr.v[VarType::ChgStnSellTr as usize].v;
                    tr.v[VarType::MvPowSatTr as usize].v = tr.v[VarType::MaxPosPowSub as usize].v
                        / z2o(tr.v[VarType::SubPowCap as usize].v);
                    tr.v[VarType::MvVspp as usize].v = tr.v[VarType::VsppMv as usize].v;
                    tr.v[VarType::HvSpp as usize].v = tr.v[VarType::SppHv as usize].v;
                    tr.v[VarType::SmallSell as usize].v = tr.v[VarType::SmallSellTr as usize].v;
                    tr.v[VarType::LargeSell as usize].v = tr.v[VarType::LargeSellTr as usize].v;
                    tr.v[VarType::UnbalPow as usize].v = tr.v[VarType::UnbalPowTr as usize].v;
                    let v = tr.v[VarType::UnbalPowTr as usize].v
                        / z2o(tr.v[VarType::PwCapTr as usize].v);
                    tr.v[VarType::CntUnbalPow as usize].v = if v > 0.5f32 { 1f32 } else { 0f32 };

                    max2.max(tr);
                }
                let _ = tx.send(max2);
            });
        }
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG2 ==== MAX2 - {se} secs");
    });
    let mut max2 = PeaAssVar::from(0u64);
    //let mut sum2 = PeaAssVar::from(0u64);
    for r in rx.iter() {
        //max2.add(&r);
        max2.max(&r);
    }
    // ----- checking

    //////////////////////////////////////
    let mut we_uc1 = PeaAssVar::from(0u64);
    for (vt, vv) in WE_UC1 {
        we_uc1.v[vt.tousz()].v = vv;
    }
    let mut we_uc2 = PeaAssVar::from(0u64);
    for (vt, vv) in WE_UC2 {
        we_uc2.v[vt.tousz()].v = vv;
    }
    let mut we_uc3 = PeaAssVar::from(0u64);
    for (vt, vv) in WE_UC3 {
        we_uc3.v[vt.tousz()].v = vv;
    }
    //let _evsc = ev_scurv();
    let yrst = ass.v(PRJ_START_YEAR) as usize;
    let yred = ass.v(PRJ_END_YEAR) as usize;
    println!("SCURVE2 st:{yrst} ed:{yred}");
    let evsc = ev_scurv2(yrst, yred);
    //let resc = re_scurv();
    let etsc = et_scurv2(yrst, yred);
    let ebsc = eb_scurv2(yrst, yred);
    //println!("evsc: {} resc: {}", evsc.len(), resc.len());

    //==== NORMALIZE 2
    let mut tras_nor = tras_raw.clone();
    {
        let tik = std::time::SystemTime::now();
        thread::scope(|s| {
            for tras_nor in tras_nor.chunks_mut(sz) {
                let max2 = max2.clone();
                s.spawn(move || {
                    for tras in tras_nor {
                        tras.nor(&max2);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG3 ==== start nor - {se} secs");
    }

    //==== EV NORMALIZE
    {
        let ass = arif.assumption();
        let mut tras_sum = tras_raw.clone();
        let tik = std::time::SystemTime::now();
        let ev_at_2023 = ass.v(EV_REG_AT_2023);
        let ev_bat_cap = ass.v(EV_BAT_CAP_MWH);
        let ev_chg_yr = ass.v(EV_TIME_FULCHG_YR);
        let ev_thb_mwh = ass.v(EV_CHG_PROF_KW) * 1000f32;
        let ev_claim = ass.v(EV_CLAIM_RATE);
        println!("======== ev23:{ev_at_2023} bat:{ev_bat_cap} chy:{ev_chg_yr}");

        let et_bat_cap = ass.v(ET_BAT_CAP_MWH);
        let et_chg_yr = ass.v(ET_TIME_FULCHG_YR);
        let et_claim = ass.v(ET_CLAIM_RATE);

        let eb_bat_cap = ass.v(EB_BAT_CAP_MWH);
        let eb_chg_yr = ass.v(EB_TIME_FULCHG_YR);
        let eb_claim = ass.v(EB_CLAIM_RATE);
        println!("===== TRK: {et_bat_cap} {et_chg_yr}  BIKE: {eb_bat_cap} {eb_chg_yr}");

        thread::scope(|s| {
            //let ac = ac.clone();
            for (trsum, trraw) in tras_sum.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let ass = ass.clone();
                let evsc = evsc.clone();
                let etsc = etsc.clone();
                let ebsc = ebsc.clone();
                let sum = sum.clone();

                s.spawn(move || {
                    //let ac = ac.clone();
                    for (tras, tras0) in trsum.iter_mut().zip(trraw.iter_mut()) {
                        tras.nor(&sum);

                        //============================== EV consumption
                        //tras0.v[VarType::NoHmChgEvTr as usize].v =
                        //    tras.v[VarType::HmChgEvTr as usize].v * 210_000f32;
                        tras0.v[VarType::NoHmChgEvTr as usize].v =
                            tras.v[VarType::HmChgEvTr as usize].v * ev_at_2023;
                        tras0.v[VarType::PowHmChgEvTr as usize].v =
                            tras0.v[VarType::NoHmChgEvTr as usize].v * 0.007f32;
                        for rt in evsc.iter() {
                            let evno =
                                tras.v[VarType::HmChgEvTr.tousz()].v * ass.v(EV_AT_2050) * rt;
                            tras0.vy[VarType::NoHmChgEvTr.tousz()].push(evno);
                            tras0.vy[VarType::PowHmChgEvTr.tousz()]
                                .push(evno * ass.v(EV_CHG_POW_KW) / 1_000f32);
                            let _evbt = evno
                                * ass.v(EV_CHG_POW_KW)
                                * ass.v(EV_DAY_CHG_HOUR)
                                * ass.v(EV_CHG_PROF_KW)
                                * 365.0
                                * ass.v(EV_CLAIM_RATE);
                            let evbt = evno * ev_bat_cap * ev_chg_yr * ev_thb_mwh * ev_claim;
                            //println!("   T:{} no:{evno} evbt:{evbt}", tras0.peano);
                            tras0.vy[VarType::FirEvChgThb.tousz()].push(evbt);
                        }
                        //tras0.sum_yr(VarType::FirEvChgThb, &ass);
                        tras0.yr_sum(VarType::FirEvChgThb, &ass);

                        //============================== EV TRUCK consumption
                        // EV truck
                        for rt in etsc.iter() {
                            let etno = tras.v[VarType::ChgEtTr.tousz()].v * ass.v(ET_AT_2050) * rt;
                            tras0.vy[VarType::NoEtTr.tousz()].push(etno);
                            let _etbt = etno
                                * ass.v(ET_CHG_POW_KW)
                                * ass.v(ET_DAY_CHG_HOUR)
                                * ass.v(EV_CHG_PROF_KW)
                                * 365.0
                                * ass.v(ET_CLAIM_RATE);
                            let etbt = etno * et_bat_cap * et_chg_yr * ev_thb_mwh * et_claim;
                            tras0.vy[VarType::FirEtChgThb.tousz()].push(etbt);
                        }
                        //tras0.sum_yr(VarType::FirEtChgThb, &ass);
                        tras0.yr_sum(VarType::FirEtChgThb, &ass);

                        // EV bike
                        for rt in ebsc.iter() {
                            let ebno = tras.v[VarType::ChgEbTr.tousz()].v * ass.v(ET_AT_2050) * rt;
                            tras0.vy[VarType::NoEtTr.tousz()].push(ebno);
                            let _ebbt = ebno
                                * ass.v(EB_CHG_POW_KW)
                                * ass.v(EB_DAY_CHG_HOUR)
                                * ass.v(EV_CHG_PROF_KW)
                                * 365.0
                                * ass.v(EB_CLAIM_RATE);
                            let ebbt = ebno * eb_bat_cap * eb_chg_yr * ev_thb_mwh * eb_claim;
                            tras0.vy[VarType::FirEbChgThb.tousz()].push(ebbt);
                        }
                        //tras0.sum_yr(VarType::FirEbChgThb, &ass);
                        tras0.yr_sum(VarType::FirEbChgThb, &ass);
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG3 ==== CALC RATIO - {se} secs");
    }

    //==== USE CASE 1
    {
        let tik = std::time::SystemTime::now();
        let mut tras_uc1 = tras_nor.clone();
        thread::scope(|s| {
            for (truc1, trraw) in tras_uc1.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_uc1 = we_uc1.clone();
                s.spawn(move || {
                    for (tras, tras0) in truc1.iter_mut().zip(trraw.iter_mut()) {
                        tras.weigh(&we_uc1);
                        tras.sum();
                        tras0.v[VarType::Uc1Val as usize].v = tras.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG3 ==== USE CASE 1 - {se} secs");
    }

    //==== USE CASE 2
    {
        let tik = std::time::SystemTime::now();
        let mut tras_uc2 = tras_nor.clone();
        thread::scope(|s| {
            for (truc2, trraw) in tras_uc2.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_uc2 = we_uc2.clone();
                s.spawn(move || {
                    for (tras, tras0) in truc2.iter_mut().zip(trraw.iter_mut()) {
                        tras.weigh(&we_uc2);
                        tras.sum();
                        tras0.v[VarType::Uc2Val as usize].v = tras.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG3 ==== USE CASE 2 - {se} secs");
    }

    //==== USE CASE 3
    {
        let tik = std::time::SystemTime::now();
        let mut tras_uc3 = tras_nor.clone();
        thread::scope(|s| {
            for (truc3, trraw) in tras_uc3.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_uc3 = we_uc3.clone();
                s.spawn(move || {
                    for (tras, tras0) in truc3.iter_mut().zip(trraw.iter_mut()) {
                        tras.weigh(&we_uc3);
                        tras.sum();
                        tras0.v[VarType::Uc3Val as usize].v = tras.res;
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG3 ==== USE CASE 3 - {se} secs");
    }

    //==== BENEFIT CALCULATION
    {
        let tik = std::time::SystemTime::now();
        thread::scope(|s| {
            for trraw in tras_raw.chunks_mut(sz) {
                let ass = ass.clone();
                s.spawn(move || {
                    for tras in trraw.iter_mut() {
                        let ary = crate::ben3::ben_bill_accu(tras, &ass);
                        tras.vy[VarType::FirBilAccu.tousz()].append(&mut ary.clone());
                        tras.yr_sum(VarType::FirBilAccu, &ass);

                        let csh = crate::ben3::ben_cash_flow(tras, &ass);
                        tras.vy[VarType::FirCashFlow.tousz()].append(&mut csh.clone());
                        tras.yr_sum(VarType::FirCashFlow, &ass);

                        let drs = crate::ben3::ben_dr_save(tras, &ass);
                        tras.vy[VarType::FirDRSave.tousz()].append(&mut drs.clone());
                        tras.yr_sum(VarType::FirDRSave, &ass);

                        let bxc = crate::ben3::ben_boxline_save(tras, &ass);
                        tras.vy[VarType::FirMetBoxSave.tousz()].append(&mut bxc.clone());
                        tras.yr_sum(VarType::FirMetBoxSave, &ass);

                        let wks = crate::ben3::ben_work_save(tras, &ass);
                        tras.vy[VarType::FirLaborSave.tousz()].append(&mut wks.clone());
                        tras.yr_sum(VarType::FirLaborSave, &ass);

                        let mts = crate::ben3::ben_sell_meter(tras, &ass);
                        tras.vy[VarType::FirMetSell.tousz()].append(&mut mts.clone());
                        tras.yr_sum(VarType::FirMetSell, &ass);

                        let ems = crate::ben3::ben_emeter(tras, &ass);
                        tras.vy[VarType::FirEMetSave.tousz()].append(&mut ems.clone());
                        tras.yr_sum(VarType::FirEMetSave, &ass);

                        let mrs = crate::ben3::ben_mt_read(tras, &ass);
                        tras.vy[VarType::FirMetReadSave.tousz()].append(&mut mrs.clone());
                        tras.yr_sum(VarType::FirMetReadSave, &ass);

                        let mds = crate::ben3::ben_mt_disconn(tras, &ass);
                        tras.vy[VarType::FirMetDisSave.tousz()].append(&mut mds.clone());
                        tras.yr_sum(VarType::FirMetDisSave, &ass);

                        let tos = crate::ben3::ben_tou_sell(tras, &ass);
                        tras.vy[VarType::FirTouSell.tousz()].append(&mut tos.clone());
                        tras.yr_sum(VarType::FirTouSell, &ass);

                        let trs = crate::ben3::ben_tou_read(tras, &ass);
                        tras.vy[VarType::FirTouReadSave.tousz()].append(&mut trs.clone());
                        tras.yr_sum(VarType::FirTouReadSave, &ass);

                        let tus = crate::ben3::ben_tou_update(tras, &ass);
                        tras.vy[VarType::FirTouUpdateSave.tousz()].append(&mut tus.clone());
                        tras.yr_sum(VarType::FirTouUpdateSave, &ass);

                        let ols = crate::ben3::ben_outage_labor(tras, &ass);
                        tras.vy[VarType::FirOutLabSave.tousz()].append(&mut ols.clone());
                        tras.yr_sum(VarType::FirOutLabSave, &ass);

                        let cps = crate::ben3::ben_reduce_complain(tras, &ass);
                        tras.vy[VarType::FirComplainSave.tousz()].append(&mut cps.clone());
                        tras.yr_sum(VarType::FirComplainSave, &ass);

                        let asv = crate::ben3::ben_asset_value(tras, &ass);
                        tras.vy[VarType::FirAssetValue.tousz()].append(&mut asv.clone());
                        tras.yr_sum(VarType::FirAssetValue, &ass);

                        let mes = crate::ben3::ben_model_entry(tras, &ass);
                        tras.vy[VarType::FirDataEntrySave.tousz()].append(&mut mes.clone());
                        tras.yr_sum(VarType::FirDataEntrySave, &ass);

                        let dum = vec![0f32; 15];
                        tras.vy[VarType::FirBatSubSave.tousz()].append(&mut dum.clone());
                        tras.yr_sum(VarType::FirBatSubSave, &ass);
                        tras.vy[VarType::FirBatSvgSave.tousz()].append(&mut dum.clone());
                        tras.yr_sum(VarType::FirBatSvgSave, &ass);
                        tras.vy[VarType::FirBatEnerSave.tousz()].append(&mut dum.clone());
                        tras.yr_sum(VarType::FirBatEnerSave, &ass);
                        tras.vy[VarType::FirBatPriceDiff.tousz()].append(&mut dum.clone());
                        tras.yr_sum(VarType::FirBatPriceDiff, &ass);

                        let nome1 = tras.v[VarType::NoMet1Ph.tousz()].v;
                        let nome3 = tras.v[VarType::NoMet3Ph.tousz()].v;
                        let notr = tras.v[VarType::NoPeaTr.tousz()].v;
                        let nobess = 0.0;
                        let bescap = 0.0;
                        let nodev = nome1 + nome3 + notr + nobess;

                        //let bescap = tras.v[VarType::PowHmChgEvTr.tousz()].v * BESS_EVPOW_MWH_MULT;
                        //tras.v[VarType::BessMWh.tousz()].v = bescap * bess_x;

                        tras.v[VarType::NoDevice.tousz()].v = nodev;

                        tras.vy[VarType::CstMet1pIns.tousz()].append(&mut cst_m1p_ins(nome1, &ass));
                        tras.yr_sum(VarType::CstMet1pIns, &ass);
                        tras.vy[VarType::CstMet3pIns.tousz()].append(&mut cst_m3p_ins(nome3, &ass));
                        tras.yr_sum(VarType::CstMet3pIns, &ass);
                        tras.vy[VarType::CstTrIns.tousz()].append(&mut cst_tr_ins(notr, &ass));
                        tras.yr_sum(VarType::CstTrIns, &ass);
                        tras.vy[VarType::CstBessIns.tousz()].append(&mut cst_bes_ins(bescap, &ass));
                        tras.yr_sum(VarType::CstBessIns, &ass);
                        tras.vy[VarType::CstPlfmIns.tousz()].append(&mut cst_plfm_ins(nodev, &ass));
                        tras.yr_sum(VarType::CstPlfmIns, &ass);
                        tras.vy[VarType::CstCommIns.tousz()].append(&mut cst_comm_ins(nodev, &ass));
                        tras.yr_sum(VarType::CstCommIns, &ass);

                        tras.vy[VarType::CstMet1pImp.tousz()].append(&mut cst_m1p_imp(nome1, &ass));
                        tras.yr_sum(VarType::CstMet1pImp, &ass);
                        tras.vy[VarType::CstMet3pImp.tousz()].append(&mut cst_m3p_imp(nome3, &ass));
                        tras.yr_sum(VarType::CstMet3pImp, &ass);
                        tras.vy[VarType::CstTrImp.tousz()].append(&mut cst_tr_imp(notr, &ass));
                        tras.yr_sum(VarType::CstTrImp, &ass);
                        tras.vy[VarType::CstBessImp.tousz()].append(&mut cst_bes_imp(bescap, &ass));
                        tras.yr_sum(VarType::CstBessImp, &ass);
                        tras.vy[VarType::CstPlfmImp.tousz()].append(&mut cst_plfm_imp(nodev, &ass));
                        tras.yr_sum(VarType::CstPlfmImp, &ass);
                        tras.vy[VarType::CstCommImp.tousz()].append(&mut cst_comm_imp(nodev, &ass));
                        tras.yr_sum(VarType::CstCommImp, &ass);

                        tras.vy[VarType::CstMet1pOp.tousz()].append(&mut cst_m1p_op(nome1, &ass));
                        tras.yr_sum(VarType::CstMet1pOp, &ass);
                        tras.vy[VarType::CstMet3pOp.tousz()].append(&mut cst_m3p_op(nome3, &ass));
                        tras.yr_sum(VarType::CstMet3pOp, &ass);
                        tras.vy[VarType::CstTrOp.tousz()].append(&mut cst_tr_op(notr, &ass));
                        tras.yr_sum(VarType::CstTrOp, &ass);
                        tras.vy[VarType::CstBessOp.tousz()].append(&mut cst_bes_op(bescap, &ass));
                        tras.yr_sum(VarType::CstBessOp, &ass);
                        tras.vy[VarType::CstPlfmOp.tousz()].append(&mut cst_plfm_op(nodev, &ass));
                        tras.yr_sum(VarType::CstPlfmOp, &ass);
                        tras.vy[VarType::CstCommOp.tousz()].append(&mut cst_comm_op(nodev, &ass));
                        tras.yr_sum(VarType::CstCommOp, &ass);

                        let sel = tras.v[VarType::AllSellTr.tousz()].v;

                        tras.vy[VarType::EirCustLossSave.tousz()]
                            .append(&mut eir_cust_loss_save(sel, &ass));
                        tras.yr_sum(VarType::EirCustLossSave, &ass);
                        tras.vy[VarType::EirConsumSave.tousz()]
                            .append(&mut eir_cust_save(sel, &ass));
                        tras.yr_sum(VarType::EirConsumSave, &ass);
                        tras.vy[VarType::EirGrnHsEmsSave.tousz()]
                            .append(&mut eir_ghg_save(sel, &ass));
                        tras.yr_sum(VarType::EirGrnHsEmsSave, &ass);
                        tras.vy[VarType::EirCustMvRev.tousz()]
                            .append(&mut eir_cust_mv_rev(sel, &ass));
                        tras.yr_sum(VarType::EirCustMvRev, &ass);
                        tras.vy[VarType::EirCustEvSave.tousz()]
                            .append(&mut eir_cust_ev_save(sel, &ass));
                        tras.yr_sum(VarType::EirCustEvSave, &ass);
                        tras.vy[VarType::EirCustEtrkSave.tousz()]
                            .append(&mut eir_cust_etruck_save(sel, &ass));
                        tras.yr_sum(VarType::EirCustEtrkSave, &ass);
                        tras.vy[VarType::EirSolaRfTopSave.tousz()]
                            .append(&mut eir_cust_solar_roof(sel, &ass));
                        tras.yr_sum(VarType::EirSolaRfTopSave, &ass);
                        tras.vy[VarType::EirEnerResvSave.tousz()]
                            .append(&mut eir_en_rev_save(sel, &ass));
                        tras.yr_sum(VarType::EirEnerResvSave, &ass);

                        ass_calc(tras, &ass).expect("?");
                    }
                });
            }
        });
        let se = tik.elapsed().unwrap().as_secs();
        println!("======== STG4 ==== COST/BENEFIT - {se} secs");
    }
    for (i, rw) in tras_raw.iter_mut().enumerate() {
        rw.ix = i;
    }

    //====== TPO CALCULATION
    let mut unb_v: Vec<_> = tras_raw
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::FirUnbSave as usize].v, i))
        .collect();
    unb_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in unb_v.iter().enumerate() {
        let me1 = tras_raw[*i].v[VarType::NoMet1Ph.tousz()].v;
        let me3 = tras_raw[*i].v[VarType::NoMet3Ph.tousz()].v;
        //let me1 = 1.0;
        //let me3 = 1.0;
        if r < tpo_no {
            tras_raw[*i].v[VarType::TpoAdd as usize].v = 1.0;
            tras_raw[*i].v[VarType::NoMet1PhSim.tousz()].v = me1;
            tras_raw[*i].v[VarType::NoMet3PhSim.tousz()].v = me3;
            tras_raw[*i].v[VarType::NoPeaTrSim.tousz()].v = 1.0;
        } else if r < tpo_no + ecu_no {
            tras_raw[*i].v[VarType::EcuAdd as usize].v = 1.0;
            tras_raw[*i].v[VarType::NoMet1PhPlc.tousz()].v = me1;
            tras_raw[*i].v[VarType::NoMet3PhPlc.tousz()].v = me3;
            tras_raw[*i].v[VarType::NoPeaTrPlc.tousz()].v = 1.0;
        } else {
            tras_raw[*i].v[VarType::NoMet1PhSim.tousz()].v = me1;
            tras_raw[*i].v[VarType::NoMet3PhSim.tousz()].v = me3;
            tras_raw[*i].v[VarType::NoPeaTrSim.tousz()].v = 1.0;
        }
    }

    Ok(tras_raw)
}

pub fn ass_rank(asss: &mut [PeaAssVar]) {
    let mut uc1_v: Vec<_> = asss
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc1Val as usize].v, i))
        .collect();
    uc1_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc1_v.iter().enumerate() {
        //asss[*i].v[VarType::Uc1Rank as usize].v = r as f32 + 1.0;
        asss[*i].v[VarType::Uc1Rank as usize].v = r as f32 + 1.0;
    }

    let mut uc2_v: Vec<_> = asss
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc2Val as usize].v, i))
        .collect();
    uc2_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc2_v.iter().enumerate() {
        asss[*i].v[VarType::Uc2Rank as usize].v = r as f32 + 1.0;
    }

    let mut uc3_v: Vec<_> = asss
        .iter()
        .enumerate()
        .map(|(i, s)| (s.v[VarType::Uc3Val as usize].v, i))
        .collect();
    uc3_v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (r, (_, i)) in uc3_v.iter().enumerate() {
        asss[*i].v[VarType::Uc3Rank as usize].v = r as f32 + 1.0;
    }
}

pub fn ass_calc(sbas: &mut PeaAssVar, ac: &Assumption) -> Result<(), Box<dyn Error>> {
    // ==========  LOSS CALCULATION
    let unb_los = sbas.v[VarType::UnbalPowLossKw.tousz()].v
        * ac.v(UNBAL_HOUR_PER_DAY)
        * ac.v(SAVE_LOSS_UNIT_PRICE)
        * ac.v(UNBAL_CALC_FACTOR)
        * 365.0;
    //let unb_los = sbas.v[VarType::UnbalPowLossKw.tousz()].v * 4.0 * 4.0;
    let mut los_sav = unb_los * ac.v(UNBAL_LOSS_CLAIM_RATE);
    //
    // transformer may die within 5 years
    // unit price for replace transformers
    // claim save ratio 0.5
    let mut tr_sav = sbas.v[VarType::CntTrSatLoss.tousz()].v / ac.v(TRANS_REPL_WITHIN_YEAR)
        * ac.v(TRANS_REPL_UNIT_PRICE)
        * ac.v(TRANS_REPL_CLAIM_RATE);
    let mut ubt_sav = sbas.v[VarType::CntTrUnbalLoss.tousz()].v / ac.v(TRANS_REPL_WITHIN_YEAR)
        * ac.v(TRANS_REPL_UNIT_PRICE)
        * ac.v(UNBAL_REPL_CLAIM_RATE);
    let mut all_sel = sbas.v[VarType::AllSellTr.tousz()].v
                    * ac.v(NON_TECH_LOSS_RATIO)
                    * 12.0 // in one year
                    * ac.v(SAVE_LOSS_UNIT_PRICE)
                    * ac.v(NOTEC_LOSS_CLAIM_RATE);
    //sbas.v[VarType::AllSellTr.tousz()].v * 12.0 * 0.9 * 4_000f32 * 0.01;

    sbas.vy[VarType::FirUnbSave.tousz()].retain(|&_| false);
    sbas.vy[VarType::FirTrSatSave.tousz()].retain(|&_| false);
    sbas.vy[VarType::FirTrPhsSatSave.tousz()].retain(|&_| false);
    sbas.vy[VarType::FirNonTechLoss.tousz()].retain(|&_| false);
    for _i in 0..15 {
        los_sav *= 1.0 + ac.v(ENERGY_GRW_RATE);
        tr_sav *= 1.0 + ac.v(ENERGY_GRW_RATE);
        ubt_sav *= 1.0 + ac.v(ENERGY_GRW_RATE);
        all_sel *= 1.0 + ac.v(ENERGY_GRW_RATE);
        //all_sel = 0.0;
        //let (los, tr, ubt, all) = if i < 3 {
        //    (0.0, 0.0, 0.0, 0.0)
        //} else {
        let (los, tr, ubt, all) = (los_sav, tr_sav, ubt_sav, all_sel);
        //};
        sbas.vy[VarType::FirUnbSave.tousz()].push(los);
        sbas.vy[VarType::FirTrSatSave.tousz()].push(tr);
        sbas.vy[VarType::FirTrPhsSatSave.tousz()].push(ubt);
        sbas.vy[VarType::FirNonTechLoss.tousz()].push(all);
    }
    sbas.yr_sum(VarType::FirUnbSave, ac);
    sbas.yr_sum(VarType::FirTrSatSave, ac);
    sbas.yr_sum(VarType::FirTrPhsSatSave, ac);
    sbas.yr_sum(VarType::FirNonTechLoss, ac);

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

    let reinv = sbas.v[VarType::CstCapEx.tousz()].v * ac.v(REINVEST_RATE);

    sbas.vy[VarType::CstReinvest.tousz()].retain(|&_| false);
    sbas.vy[VarType::CstReinvest.tousz()].append(&mut cst_reinvest(reinv, ac));
    //sbas.v[VarType::CstReinvest.tousz()].v = sbas.vy[VarType::CstReinvest.tousz()].iter().sum();
    sbas.yr_sum(VarType::CstReinvest, ac);

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
    sbas.vy[VarType::FirCstRate.tousz()] = fir_cpx_opx.clone();
    sbas.vy[VarType::EirCstRate.tousz()] = eir_cpx_opx.clone();

    /*
    let fir_cpx_opx = fir_cpx_opx
        .iter()
        .filter(|n| !n.is_nan())
        .cloned()
        .collect::<Vec<_>>();
    let s0 = fir_cpx_opx.iter().sum::<f32>();

    let firr = if fir_cpx_opx.len() == 15 && s0 > 0f32 {
        let guess = Some(0.);
        let fir: Vec<f64> = fir_cpx_opx.iter().map(|n| *n as f64).collect();
        if let Ok(firr) = financial::irr(&fir, guess) {
            firr * 100.0f64
        } else {
            0f64
        }
    } else {
        0f64
    };
    */
    //sbas.v[VarType::FirCstRate.tousz()].v = firr as f32;
    sbas.v[VarType::FirCstRate.tousz()].v = if sbas.v[VarType::CstCapOpEx.tousz()].v > 0.0 {
        sbas.v[VarType::FirSum.tousz()].v / sbas.v[VarType::CstCapOpEx.tousz()].v
    } else {
        0.0
    };

    let eir_cpx_opx = eir_cpx_opx
        .iter()
        .filter(|n| !n.is_nan())
        .cloned()
        .collect::<Vec<_>>();
    let s0 = eir_cpx_opx.iter().sum::<f32>();
    let eirr = if eir_cpx_opx.len() == 15 && s0 > 0f32 {
        let guess = Some(0.);
        let eir: Vec<f64> = eir_cpx_opx.iter().map(|n| *n as f64).collect();
        if let Ok(eirr) = financial::irr(&eir, guess) {
            eirr * 100.0f64
        } else {
            0f64
        }
    } else {
        0f64
    };
    //println!("FIRR: {}", firr);
    sbas.v[VarType::EirCstRate.tousz()].v = eirr as f32;

    Ok(())
}

//================================================
//================================================
