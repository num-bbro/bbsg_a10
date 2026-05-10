use crate::asm::ASM::*;
use crate::dcl::ProcEngine;
use crate::dcl::*;
use crate::p08::p08_class_val;
use crate::p08::ProfType;
use crate::utl::mon_kwh_2_kw;
use crate::utl::trf_kva_2_kw;
use crate::utl::*;
use regex::Regex;
use sglib04::geo4::PowerProdType;
use std::collections::HashMap;
use std::error::Error;

//use crate::utl4::make_archi;
use crate::utl4::Archi;

use crate::cst2::cst_bes_imp;
use crate::cst2::cst_bes_ins;
use crate::cst2::cst_bes_op;
use crate::cst2::cst_comm_imp;
use crate::cst2::cst_comm_ins;
use crate::cst2::cst_comm_op;
use crate::cst2::cst_m1p_imp;
use crate::cst2::cst_m1p_ins;
use crate::cst2::cst_m1p_op;
use crate::cst2::cst_m3p_imp;
use crate::cst2::cst_m3p_ins;
use crate::cst2::cst_m3p_op;
use crate::cst2::cst_plfm_imp;
use crate::cst2::cst_plfm_ins;
use crate::cst2::cst_plfm_op;
use crate::cst2::cst_tr_imp;
use crate::cst2::cst_tr_ins;
use crate::cst2::cst_tr_op;
use crate::cst2::eir_cust_etruck_save;
use crate::cst2::eir_cust_ev_save;
use crate::cst2::eir_cust_loss_save;
use crate::cst2::eir_cust_mv_rev;
use crate::cst2::eir_cust_save;
use crate::cst2::eir_cust_solar_roof;
use crate::cst2::eir_en_rev_save;
use crate::cst2::eir_ghg_save;
use crate::stx2::ass_calc;
use std::sync::Arc;
use std::sync::Mutex;

pub const BESS_EVPOW_MWH_MULT: f32 = 1.0;
use std::sync::mpsc;
use std::thread;

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

pub fn load(_coreno: usize, _vwid: String) -> Result<(), Box<dyn Error>> {
    let dnm = crate::dcl::get_dirnm();
    let fnm = format!("{dnm}/all-rw4.bin");
    let Ok(buf) = std::fs::read(&fnm) else {
        return Err(format!("NO {fnm} file:").into());
    };
    let Ok((assv0, _)): Result<(Vec<PeaAssVar>, usize), _> =
        bincode::decode_from_slice(&buf[..], bincode::config::standard())
    else {
        return Err("Failed to decode rw3:".into());
    };
    println!("rw {}", assv0.len());

    Ok(())
}

/// read 000_pea.bin
/// read SSS.bin
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

pub fn stage_02_1(_coreno: usize, ac: &Archi) -> Result<Vec<PeaAssVar>, Box<dyn Error>> {
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

    let dnm = ac.t(OUTDIR);
    let fnm = format!("{dnm}/000_pea.bin");
    println!("fnm:{fnm}");
    let buf = std::fs::read(fnm).unwrap();
    let (mut pea, _): (Pea, usize) =
        bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();

    //===== PROCESS EACH AREA
    std::thread::scope(|s| {
        for (aid, ar) in pea.aream.iter_mut() {
            //for id in aids {
            let id = aid.to_string();
            let ac = ac.clone();

            let c_e0 = a_e0.clone();
            let c_fd2fd = a_fd2fd.clone();
            let c_assv = a_assv.clone();
            let ac = ac.clone();
            let dnm = dnm.clone();

            //let id = id.to_string();
            //let aream = pea.aream.clone();
            let _handle = s.spawn(move || {
                //let Some(ar) = aream.get(&aid) else {
                //    return;
                //};
                //println!("ar:{}", ar.arid);
                let eg = ProcEngine::prep3(&id);
                //==== AMPHO INIT
                let mut am_dn = HashMap::<String, (f32, f32)>::new();
                for dn in &eg.amps {
                    let key = dn.key.to_string();
                    if let Some((_po, aa)) = am_dn.get_mut(&key) {
                        *aa += dn.area;
                    } else {
                        am_dn.insert(key, (dn.popu, dn.area));
                    }
                }
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
                // province loop1
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
                                    if let Ok(lpf) = p08_class_val(lpv) {
                                        if lpf.lp_type == ProfType::SolarPower
                                            || lpf.lp_type == ProfType::SolarNight
                                        {
                                            solar = -lpf.sol_en.unwrap_or(0f32);
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
                            let mut grw = ac.v(EN_AVG_GRW_RATE);
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
                                    if grw2 > grw && grw2 < ac.v(EN_MAX_GRW_RATE) {
                                        grw = grw2;
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
                                        //print!("_{}", met.kwh15);
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
                                let v_loss = v_al_kw * ac.v(TRF_LOSS_RATIO);
                                let v_unba = v_loss * ac.v(TRF_UNBAL_K) * v_ph_rt * v_ph_rt;
                                let v_unb_sat = v_ph_mx / z2o(vt05);
                                let v_unb_cnt = if v_unb_sat >= ac.v(TRF_UNBAL_CNT_RATE) {
                                    1f32
                                } else {
                                    0f32
                                };
                                let v_max_sat = v_all_p / z2o(vt05);
                                let v_max_cnt =
                                    if v_unb_cnt == 0f32 && v_max_sat >= ac.v(TRF_UNBAL_CNT_RATE) {
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
                                tr_as.v[VarType::EnGrowth as usize].v = grw;
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
                        /*
                        let bin: Vec<u8> =
                            bincode::encode_to_vec(&s_tr_ass, bincode::config::standard()).unwrap();
                        std::fs::write(format!("{dnm}/{sid}-raw.bin"), bin).unwrap();
                        */
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

pub fn stage_02_b(coreno: usize, ac: &Archi, assrw1: Vec<PeaAssVar>) -> Result<(), Box<dyn Error>> {
    //  Assumption Constants
    let me_x = ac.v(METER_NO_MULTIPLY);
    let bess_x = ac.v(BESS_EVCAP_MULTIPLY);
    println!("============ me_x: {me_x} bess_x : {bess_x}");

    //let cn = 10;
    //let sz = (assrw1.len() + cn - 1) / cn;
    let sz = assrw1.len().div_ceil(coreno);
    let dnm = ac.t(OUTDIR);

    let tpo_no = 20_000;
    let ecu_no = 10_000;
    let _svg_no = 300;

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

    let _tpa_day_hours = ac.v(TPA_DAY_HOURS);
    let _tpa_price_thb = ac.v(TPA_PRICE_THB);
    let _tpa_ben_claim = ac.v(TPA_BEN_CLAIM);
    let _tpa_year_days = ac.v(TPA_YEAR_DAYS);
    let tpafcs = ac.ve(TPA_FORECAST)?;
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
    let _resc = re_scurv();
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
        let mut tras_sum = tras_raw.clone();
        let tik = std::time::SystemTime::now();
        thread::scope(|s| {
            for (trsum, trraw) in tras_sum.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let ac = ac.clone();
                let evsc = evsc.clone();
                let etsc = etsc.clone();
                let ebsc = ebsc.clone();
                let sum = sum.clone();
                s.spawn(move || {
                    let ac = ac.clone();
                    for (tras, tras0) in trsum.iter_mut().zip(trraw.iter_mut()) {
                        tras.nor(&sum);

                        //============================== EV consumption
                        tras0.v[VarType::NoHmChgEvTr as usize].v =
                            tras.v[VarType::HmChgEvTr as usize].v * 210_000f32;
                        tras0.v[VarType::PowHmChgEvTr as usize].v =
                            tras0.v[VarType::NoHmChgEvTr as usize].v * 0.007f32;
                        for rt in evsc.iter() {
                            let evno = tras.v[VarType::HmChgEvTr.tousz()].v * ac.v(EV_AT_2050) * rt;
                            tras0.vy[VarType::NoHmChgEvTr.tousz()].push(evno);
                            tras0.vy[VarType::PowHmChgEvTr.tousz()]
                                .push(evno * ac.v(EV_CHG_POW_KW) / 1_000f32);
                            let evbt = evno
                                * ac.v(EV_CHG_POW_KW)
                                * ac.v(EV_DAY_CHG_HOUR)
                                * ac.v(EV_CHG_PROF_KW)
                                * 365.0
                                * ac.v(EV_CLAIM_RATE);
                            tras0.vy[VarType::FirEvChgThb.tousz()].push(evbt);
                        }
                        tras0.sum_yr(VarType::FirEvChgThb, &ac);

                        //============================== EV TRUCK consumption
                        // EV truck
                        for rt in etsc.iter() {
                            let etno = tras.v[VarType::ChgEtTr.tousz()].v * ac.v(ET_AT_2050) * rt;
                            tras0.vy[VarType::NoEtTr.tousz()].push(etno);
                            let etbt = etno
                                * ac.v(ET_CHG_POW_KW)
                                * ac.v(ET_DAY_CHG_HOUR)
                                * ac.v(EV_CHG_PROF_KW)
                                * 365.0
                                * ac.v(ET_CLAIM_RATE);
                            tras0.vy[VarType::FirEtChgThb.tousz()].push(etbt);
                        }
                        tras0.sum_yr(VarType::FirEtChgThb, &ac);

                        // EV bike
                        for rt in ebsc.iter() {
                            let ebno = tras.v[VarType::ChgEbTr.tousz()].v * ac.v(ET_AT_2050) * rt;
                            tras0.vy[VarType::NoEtTr.tousz()].push(ebno);
                            let ebbt = ebno
                                * ac.v(EB_CHG_POW_KW)
                                * ac.v(EB_DAY_CHG_HOUR)
                                * ac.v(EV_CHG_PROF_KW)
                                * 365.0
                                * ac.v(EB_CLAIM_RATE);
                            tras0.vy[VarType::FirEbChgThb.tousz()].push(ebbt);
                        }
                        tras0.sum_yr(VarType::FirEbChgThb, &ac);
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
                let ac = ac.clone();
                s.spawn(move || {
                    for tras in trraw.iter_mut() {
                        let ary = crate::ben2::ben_bill_accu(tras, &ac);
                        tras.vy[VarType::FirBilAccu.tousz()].append(&mut ary.clone());
                        tras.sum_yr(VarType::FirBilAccu, &ac);

                        let csh = crate::ben2::ben_cash_flow(tras, &ac);
                        tras.vy[VarType::FirCashFlow.tousz()].append(&mut csh.clone());
                        tras.sum_yr(VarType::FirCashFlow, &ac);

                        let drs = crate::ben2::ben_dr_save(tras, &ac);
                        tras.vy[VarType::FirDRSave.tousz()].append(&mut drs.clone());
                        tras.sum_yr(VarType::FirDRSave, &ac);

                        let bxc = crate::ben2::ben_boxline_save(tras, &ac);
                        tras.vy[VarType::FirMetBoxSave.tousz()].append(&mut bxc.clone());
                        tras.sum_yr(VarType::FirMetBoxSave, &ac);

                        let wks = crate::ben2::ben_work_save(tras, &ac);
                        tras.vy[VarType::FirLaborSave.tousz()].append(&mut wks.clone());
                        tras.sum_yr(VarType::FirLaborSave, &ac);

                        let mts = crate::ben2::ben_sell_meter(tras, &ac);
                        tras.vy[VarType::FirMetSell.tousz()].append(&mut mts.clone());
                        tras.sum_yr(VarType::FirMetSell, &ac);

                        let ems = crate::ben2::ben_emeter(tras, &ac);
                        tras.vy[VarType::FirEMetSave.tousz()].append(&mut ems.clone());
                        tras.sum_yr(VarType::FirEMetSave, &ac);

                        let mrs = crate::ben2::ben_mt_read(tras, &ac);
                        tras.vy[VarType::FirMetReadSave.tousz()].append(&mut mrs.clone());
                        tras.sum_yr(VarType::FirMetReadSave, &ac);

                        let mds = crate::ben2::ben_mt_disconn(tras, &ac);
                        tras.vy[VarType::FirMetDisSave.tousz()].append(&mut mds.clone());
                        tras.sum_yr(VarType::FirMetDisSave, &ac);

                        let tos = crate::ben2::ben_tou_sell(tras, &ac);
                        tras.vy[VarType::FirTouSell.tousz()].append(&mut tos.clone());
                        tras.sum_yr(VarType::FirTouSell, &ac);

                        let trs = crate::ben2::ben_tou_read(tras, &ac);
                        tras.vy[VarType::FirTouReadSave.tousz()].append(&mut trs.clone());
                        tras.sum_yr(VarType::FirTouReadSave, &ac);

                        let tus = crate::ben2::ben_tou_update(tras, &ac);
                        tras.vy[VarType::FirTouUpdateSave.tousz()].append(&mut tus.clone());
                        tras.sum_yr(VarType::FirTouUpdateSave, &ac);

                        let ols = crate::ben2::ben_outage_labor(tras, &ac);
                        tras.vy[VarType::FirOutLabSave.tousz()].append(&mut ols.clone());
                        tras.sum_yr(VarType::FirOutLabSave, &ac);

                        let cps = crate::ben2::ben_reduce_complain(tras, &ac);
                        tras.vy[VarType::FirComplainSave.tousz()].append(&mut cps.clone());
                        tras.sum_yr(VarType::FirComplainSave, &ac);

                        let asv = crate::ben2::ben_asset_value(tras, &ac);
                        tras.vy[VarType::FirAssetValue.tousz()].append(&mut asv.clone());
                        tras.sum_yr(VarType::FirAssetValue, &ac);

                        let mes = crate::ben2::ben_model_entry(tras, &ac);
                        tras.vy[VarType::FirDataEntrySave.tousz()].append(&mut mes.clone());
                        tras.sum_yr(VarType::FirDataEntrySave, &ac);

                        let dum = vec![0f32; 15];
                        tras.vy[VarType::FirBatSubSave.tousz()].append(&mut dum.clone());
                        tras.sum_yr(VarType::FirBatSubSave, &ac);
                        tras.vy[VarType::FirBatSvgSave.tousz()].append(&mut dum.clone());
                        tras.sum_yr(VarType::FirBatSvgSave, &ac);
                        tras.vy[VarType::FirBatEnerSave.tousz()].append(&mut dum.clone());
                        tras.sum_yr(VarType::FirBatEnerSave, &ac);
                        tras.vy[VarType::FirBatPriceDiff.tousz()].append(&mut dum.clone());
                        tras.sum_yr(VarType::FirBatPriceDiff, &ac);

                        let nome1 = tras.v[VarType::NoMet1Ph.tousz()].v;
                        let nome3 = tras.v[VarType::NoMet3Ph.tousz()].v;
                        let notr = tras.v[VarType::NoPeaTr.tousz()].v;
                        let nobess = 0.0;
                        //let bescap = 0.0;
                        let nodev = nome1 + nome3 + notr + nobess;

                        let bescap = tras.v[VarType::PowHmChgEvTr.tousz()].v * BESS_EVPOW_MWH_MULT;
                        tras.v[VarType::BessMWh.tousz()].v = bescap * bess_x;

                        tras.v[VarType::NoDevice.tousz()].v = nodev;

                        tras.vy[VarType::CstMet1pIns.tousz()].append(&mut cst_m1p_ins(nome1, &ac));
                        tras.sum_yr(VarType::CstMet1pIns, &ac);
                        tras.vy[VarType::CstMet3pIns.tousz()].append(&mut cst_m3p_ins(nome3, &ac));
                        tras.sum_yr(VarType::CstMet3pIns, &ac);
                        tras.vy[VarType::CstTrIns.tousz()].append(&mut cst_tr_ins(notr, &ac));
                        tras.sum_yr(VarType::CstTrIns, &ac);
                        tras.vy[VarType::CstBessIns.tousz()].append(&mut cst_bes_ins(bescap, &ac));
                        tras.sum_yr(VarType::CstBessIns, &ac);
                        tras.vy[VarType::CstPlfmIns.tousz()].append(&mut cst_plfm_ins(nodev, &ac));
                        tras.sum_yr(VarType::CstPlfmIns, &ac);
                        tras.vy[VarType::CstCommIns.tousz()].append(&mut cst_comm_ins(nodev, &ac));
                        tras.sum_yr(VarType::CstCommIns, &ac);

                        tras.vy[VarType::CstMet1pImp.tousz()].append(&mut cst_m1p_imp(nome1, &ac));
                        tras.sum_yr(VarType::CstMet1pImp, &ac);
                        tras.vy[VarType::CstMet3pImp.tousz()].append(&mut cst_m3p_imp(nome3, &ac));
                        tras.sum_yr(VarType::CstMet3pImp, &ac);
                        tras.vy[VarType::CstTrImp.tousz()].append(&mut cst_tr_imp(notr, &ac));
                        tras.sum_yr(VarType::CstTrImp, &ac);
                        tras.vy[VarType::CstBessImp.tousz()].append(&mut cst_bes_imp(bescap, &ac));
                        tras.sum_yr(VarType::CstBessImp, &ac);
                        tras.vy[VarType::CstPlfmImp.tousz()].append(&mut cst_plfm_imp(nodev, &ac));
                        tras.sum_yr(VarType::CstPlfmImp, &ac);
                        tras.vy[VarType::CstCommImp.tousz()].append(&mut cst_comm_imp(nodev, &ac));
                        tras.sum_yr(VarType::CstCommImp, &ac);

                        tras.vy[VarType::CstMet1pOp.tousz()].append(&mut cst_m1p_op(nome1, &ac));
                        tras.sum_yr(VarType::CstMet1pOp, &ac);
                        tras.vy[VarType::CstMet3pOp.tousz()].append(&mut cst_m3p_op(nome3, &ac));
                        tras.sum_yr(VarType::CstMet3pOp, &ac);
                        tras.vy[VarType::CstTrOp.tousz()].append(&mut cst_tr_op(notr, &ac));
                        tras.sum_yr(VarType::CstTrOp, &ac);
                        tras.vy[VarType::CstBessOp.tousz()].append(&mut cst_bes_op(bescap, &ac));
                        tras.sum_yr(VarType::CstBessOp, &ac);
                        tras.vy[VarType::CstPlfmOp.tousz()].append(&mut cst_plfm_op(nodev, &ac));
                        tras.sum_yr(VarType::CstPlfmOp, &ac);
                        tras.vy[VarType::CstCommOp.tousz()].append(&mut cst_comm_op(nodev, &ac));
                        tras.sum_yr(VarType::CstCommOp, &ac);

                        let sel = tras.v[VarType::AllSellTr.tousz()].v;

                        tras.vy[VarType::EirCustLossSave.tousz()]
                            .append(&mut eir_cust_loss_save(sel, &ac));
                        tras.sum_yr(VarType::EirCustLossSave, &ac);
                        tras.vy[VarType::EirConsumSave.tousz()]
                            .append(&mut eir_cust_save(sel, &ac));
                        tras.sum_yr(VarType::EirConsumSave, &ac);
                        tras.vy[VarType::EirGrnHsEmsSave.tousz()]
                            .append(&mut eir_ghg_save(sel, &ac));
                        tras.sum_yr(VarType::EirGrnHsEmsSave, &ac);
                        tras.vy[VarType::EirCustMvRev.tousz()]
                            .append(&mut eir_cust_mv_rev(sel, &ac));
                        tras.sum_yr(VarType::EirCustMvRev, &ac);
                        tras.vy[VarType::EirCustEvSave.tousz()]
                            .append(&mut eir_cust_ev_save(sel, &ac));
                        tras.sum_yr(VarType::EirCustEvSave, &ac);
                        tras.vy[VarType::EirCustEtrkSave.tousz()]
                            .append(&mut eir_cust_etruck_save(sel, &ac));
                        tras.sum_yr(VarType::EirCustEtrkSave, &ac);
                        tras.vy[VarType::EirSolaRfTopSave.tousz()]
                            .append(&mut eir_cust_solar_roof(sel, &ac));
                        tras.sum_yr(VarType::EirSolaRfTopSave, &ac);
                        tras.vy[VarType::EirEnerResvSave.tousz()]
                            .append(&mut eir_en_rev_save(sel, &ac));
                        tras.sum_yr(VarType::EirEnerResvSave, &ac);

                        ass_calc(tras, &ac).expect("?");
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
    {
        let tik = std::time::SystemTime::now();
        let fnm = format!("{dnm}/all-rw4.bin");
        let bin: Vec<u8> = bincode::encode_to_vec(&tras_raw, bincode::config::standard()).unwrap();
        std::fs::write(fnm, bin).unwrap();
        let se = tik.elapsed().unwrap().as_secs();
        println!("SAVE ALL:{} - {se}sec", tras_raw.len());
    }

    Ok(())
}
