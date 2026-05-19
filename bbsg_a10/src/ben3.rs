use crate::asm::ASM::*;
//use crate::dcl::Pan;
use crate::dcl::PeaAssVar;
use crate::dcl::VarType;
use crate::utl6::Assumption;
use num::pow::Pow;
//use sglib03::prc4::SubYearBenInfo;

use sglib03::prc2::PowerCalc;

pub fn ben_bill_accu(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = tras.v[VarType::AllSellTr.tousz()].v
            * ac.v(SMETER_ACCU_IMPRV)
            * ac.v(SMETER_BILL_IMPRV);
        let be = be * 30.0;
        let be = be * ac.v(UNIT_PRICE);
        let be: f32 = be / Pow::pow(1.0 + ac.v(ECON_GRW_RATE), y);
        let be: f32 = be * Pow::pow(1.0 + ac.v(ENGY_GRW_RATE), y);
        let be = be * 0.1;
        proj.push(be);
    }
    proj
}

pub fn ben_cash_flow(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    let al0 = tras.v[VarType::AllSellTr.tousz()].v;
    let dl_80 = ac.v(CASH_DAY_DELAY_80);
    let dl_20 = ac.v(CASH_DAY_DELAY_20);
    let dl_0 = dl_80 * 0.8 + dl_20 * 0.2; // average days delay of cash flow
    let dl_d = dl_0 - ac.v(CASH_DAY_DELAY_SMART); // diff days improved
    let dl_m1 = al0 * ac.v(UNIT_PRICE) / 365.0 * dl_d * ac.v(CASH_FLOW_COST);
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = dl_m1 as f64;
        // adjust
        let be = be * 40f64;
        let be = be / Pow::pow(1f64 + ac.v(ECON_GRW_RATE) as f64, y as f64);
        let be = be * Pow::pow(1f64 + ac.v(ENGY_GRW_RATE) as f64, y as f64);
        //print!(" {}-{be:.2}", y + 2028);
        proj.push(be as f32);
    }
    proj
}

pub fn ben_dr_save(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    //let cap1 = 80_000_000f64 / 22_000_000f64;
    //let cap2 = 20_000_000f64 / 22_000_000f64;
    //print!("====  Demand Response ");
    let mt_1_ph = tras.v[VarType::NoMet1Ph.tousz()].v * ac.v(DR_DEV_PLAN_RATE);
    let mt_3_ph = tras.v[VarType::NoMet3Ph.tousz()].v * ac.v(DR_DEV_PLAN_RATE);
    let cap3 = mt_1_ph * ac.v(MET_1PH_COST);
    let cap4 = mt_3_ph * ac.v(MET_3PH_COST);
    let opx1 = cap3 * 0.005;
    let opx2 = cap4 * 0.005;
    let opx3 = (mt_1_ph + mt_3_ph) * 55.0 * 12.0;
    let opx4 = cap3 * 0.05;
    let opx5 = cap4 * 0.05;
    //let mut proj = Vec::<(u32, f32)>::new();
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = if y == 0 { cap3 + cap4 } else { 0.0 };
        let be = be + opx1 + opx2 + opx3 + opx4 + opx5;
        // adjust
        let be = be * 1.1;
        let be = be / Pow::pow(1.0 + ac.v(ECON_GRW_RATE), y);
        let be = be * Pow::pow(1.0 + ac.v(ENGY_GRW_RATE), y);
        proj.push(be);
    }
    proj
}

pub fn ben_boxline_save(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    //print!("====  BOX : ");
    let boxcnt = tras.v[VarType::NoMet1Ph.tousz()].v + tras.v[VarType::NoMet3Ph.tousz()].v;
    let boxcnt = boxcnt * ac.v(BOX_LINE_NEED_RATE);
    let boxex = boxcnt * ac.v(BOX_LINE_UNIT_COST);
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = boxex;
        let be = be / Pow::pow(1.0 + ac.v(ECON_GRW_RATE), y);
        let be = be * Pow::pow(1.0 + ac.v(ENGY_GRW_RATE), y);
        proj.push(be);
    }
    proj
}

pub fn ben_work_save(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    let wk_cnt = tras.v[VarType::NoMet1Ph.tousz()].v + tras.v[VarType::NoMet3Ph.tousz()].v;
    let wk_cnt = wk_cnt / ac.v(METER_PER_WORKER);
    let mn_exp =
        ac.v(WORKER_MONTH_SALARY) * (1.0 + ac.v(WORKER_SAVING_RATE) + ac.v(WORKER_SOC_SEC_RATE));
    let yr_exp = mn_exp * 12.0 + ac.v(WORKER_MONTH_SALARY) * ac.v(WORKER_BONUS_MONTH);
    let yr_exp = yr_exp * wk_cnt;
    //print!(" mn:{mn_exp} yr:{yr_exp}");
    let wk_redu = yr_exp * ac.v(WORKER_REDUCE_RATE);
    //print!(" rd:{wk_redu}");
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = wk_redu;
        let be = be / Pow::pow(1.0 + ac.v(ECON_GRW_RATE), y);
        let be = be * Pow::pow(1.0 + ac.v(SALARY_INCR_RATE), y);
        //print!(" {}-{be:.2}", y + 2028);
        proj.push(be);
    }
    proj
}

pub fn ben_sell_meter(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    //print!("====  SELL METER");
    let m1p = tras.v[VarType::NoMet1Ph.tousz()].v * ac.v(METER_SELLABLE_RATE);
    let m3p = tras.v[VarType::NoMet1Ph.tousz()].v * ac.v(METER_SELLABLE_RATE);
    let m1p_s = m1p * ac.v(M1P_SELL_PRICE);
    let m3p_s = m3p * ac.v(M3P_SELL_PRICE);
    let m1p_y = m1p_s / 12.0;
    let m3p_y = m3p_s / 12.0;
    //let mut proj = Vec::<(u32, f32)>::new();
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for _y in 0..15 {
        let be = m1p_y + m3p_y;
        proj.push(be);
    }
    proj
}

pub fn ben_emeter(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    //print!("====  EMETER");
    let m1_cnt = tras.v[VarType::NoMet1Ph.tousz()].v * ac.v(EMTR_CNT_RATIO);
    let m3_cnt = tras.v[VarType::NoMet3Ph.tousz()].v * ac.v(EMTR_CNT_RATIO);
    let m1_sw_c = m1_cnt * ac.v(EMTR_SWAP_RATE);
    let m3_sw_c = m3_cnt * ac.v(EMTR_SWAP_RATE);
    let m1_sw_e = m1_sw_c * (ac.v(EMTR_1P_COST) + ac.v(EMTR_1P_SWAP));
    let m3_sw_e = m3_sw_c * (ac.v(EMTR_3P_COST) + ac.v(EMTR_3P_SWAP));
    let m1_rp_c = m1_cnt * ac.v(EMTR_REPL_RATE);
    let m3_rp_c = m3_cnt * ac.v(EMTR_REPL_RATE);
    let m1_rp_e = m1_rp_c * (ac.v(EMTR_1P_COST) + ac.v(EMTR_1P_REPL));
    let m3_rp_e = m3_rp_c * (ac.v(EMTR_3P_COST) + ac.v(EMTR_3P_REPL));
    let ex = m1_sw_e + m3_sw_e + m1_rp_e + m3_rp_e;
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = ex;
        let be = be * Pow::pow(1.0 + ac.v(EMTR_COST_UP), y);
        proj.push(be);
    }
    proj
}

pub fn ben_mt_read(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    //print!("====  READING");
    let m1_rd = tras.v[VarType::NoMet1Ph.tousz()].v * ac.v(MT_READ_COST) * 12.0;
    let m3_rd = tras.v[VarType::NoMet3Ph.tousz()].v * ac.v(MT_READ_COST) * 12.0;
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = m1_rd + m3_rd;
        let be = be * Pow::pow(1.0 + ac.v(READ_COST_UP), y);
        proj.push(be);
    }
    proj
}

pub fn ben_mt_disconn(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    let m1_cn = tras.v[VarType::NoMet1Ph.tousz()].v * ac.v(M1_DISCON_RATE);
    let m3_cn = tras.v[VarType::NoMet3Ph.tousz()].v * ac.v(M3_DISCON_RATE);
    let m1_ex = m1_cn * ac.v(M1_DISCON_COST);
    let m3_ex = m3_cn * ac.v(M3_DISCON_COST);

    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = m1_ex + m3_ex;
        let be = be * 200.0;
        let be = be * Pow::pow(1.0 + ac.v(DISCON_COST_UP), y);
        proj.push(be);
    }
    proj
}

pub fn ben_tou_sell(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    //print!("====  SELL METER");
    let m1p = tras.v[VarType::NoMet1Ph.tousz()].v * ac.v(TOU_METER_RATIO) * ac.v(TOU_SELLABLE_RATE);
    let m3p = tras.v[VarType::NoMet3Ph.tousz()].v * ac.v(TOU_METER_RATIO) * ac.v(TOU_SELLABLE_RATE);
    let m1p_s = m1p * ac.v(TOU_1P_SELL_PRICE);
    let m3p_s = m3p * ac.v(TOU_3P_SELL_PRICE);
    let m1p_y = m1p_s / 12.0;
    let m3p_y = m3p_s / 12.0;
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for _y in 0..15 {
        let be = m1p_y + m3p_y;
        proj.push(be);
    }
    proj
}

pub fn ben_tou_read(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    let m1p = tras.v[VarType::NoMet1Ph.tousz()].v * ac.v(TOU_METER_RATIO) * 12.0;
    let m3p = tras.v[VarType::NoMet3Ph.tousz()].v * ac.v(TOU_METER_RATIO) * 12.0;
    let m1_rd = m1p * ac.v(TOU_READ_COST);
    let m3_rd = m3p * ac.v(TOU_READ_COST);
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = m1_rd + m3_rd;
        let be = be * Pow::pow(1.0 + ac.v(TOU_COST_UP), y);
        proj.push(be);
    }
    proj
}

pub fn ben_tou_update(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    let m1p = tras.v[VarType::NoMet1Ph.tousz()].v * ac.v(TOU_METER_RATIO) * 12.0;
    let m3p = tras.v[VarType::NoMet3Ph.tousz()].v * ac.v(TOU_METER_RATIO) * 12.0;
    let m1_rd = m1p * ac.v(TOU_UPDATE_COST);
    let m3_rd = m3p * ac.v(TOU_UPDATE_COST);
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = m1_rd + m3_rd;
        let be = be * Pow::pow(1.0 + ac.v(TOU_COST_UP), y);
        proj.push(be);
    }
    proj
}

pub fn ben_outage_labor(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    //print!("====  OUTAGE LABOR");
    let hr = tras.v[VarType::NoMet1Ph.tousz()].v + tras.v[VarType::NoMet3Ph.tousz()].v;
    let hr = hr * ac.v(OUT_MT_HOUR_YEAR);
    let ex = hr * ac.v(LABOR_COST_HOUR) * 5.0;
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = ex;
        let be = be * Pow::pow(1.0 + ac.v(ENGY_GRW_RATE), y);
        proj.push(be);
    }
    proj
}

// FirComplainSave
pub fn ben_reduce_complain(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    let hr = tras.v[VarType::NoMet1Ph.tousz()].v + tras.v[VarType::NoMet3Ph.tousz()].v;
    let ex = hr * ac.v(CALL_CENTER_COST_MT);
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = ex;
        let be = be * Pow::pow(1.0 + ac.v(CALL_CENTER_COST_UP), y);
        proj.push(be);
    }
    proj
}

//FirAssetValue
pub fn ben_asset_value(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    let m1i = tras.v[VarType::NoMet1Ph.tousz()].v * ac.v(M1P_COST);
    let m3i = tras.v[VarType::NoMet3Ph.tousz()].v * ac.v(M3P_COST);
    let txi = tras.v[VarType::NoPeaTr.tousz()].v * ac.v(TRX_COST);
    let esi = 0f32;
    let ass = (m1i + m3i + txi + esi) * ac.v(ASSET_WORTH_RATIO);
    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for _y in 0..11 {
        proj.push(0f32);
    }
    proj.push(ass);
    proj
}

pub fn ben_model_entry(tras: &PeaAssVar, ac: &Assumption) -> Vec<f32> {
    let m1i = tras.v[VarType::NoMet1Ph.tousz()].v;
    let m3i = tras.v[VarType::NoMet3Ph.tousz()].v;
    let txi = tras.v[VarType::NoPeaTr.tousz()].v;
    let cnt = m1i + m3i + txi;
    let ent_cn = cnt * ac.v(MODEL_ENTRY_RATIO);
    let ent_ex = ent_cn * ac.v(MODEL_ENTRY_COST);

    //let mut proj = vec![0.0, 0.0, 0.0];
    let mut proj = vec![];
    for y in 0..15 {
        let be = ent_ex;
        let be = be * Pow::pow(1.0 + ac.v(CALL_CENTER_COST_UP), y);
        proj.push(be);
    }
    proj
}

//use crate::dcl::PeaSub;
use crate::dcl::PeaSubEx1;
use crate::utl8::SubRpfOvl;
use fast_math::log2_raw;
use std::collections::HashMap;

pub fn ben_bess_calc(
    sb: &PeaSubEx1,
    sbas: &PeaAssVar,
    ac: &Assumption,
    rpfm: &HashMap<String, SubRpfOvl>,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, f32) {
    let mut sub_save = vec![0.0, 0.0, 0.0];
    let mut svg_save = vec![0.0, 0.0, 0.0];
    let mut dif_save = vec![0.0, 0.0, 0.0];
    let mut eng_save = vec![0.0, 0.0, 0.0];
    // ==============================================
    // ==============================================
    // ======= BEGIN =======
    //let grw = gr;
    //let pwmx = pwx;
    let grw = ac.v(EN_AVG_GRW_RATE);
    //let grw = sbas.v[VarType::EnGrowth.tousz()].v;
    let pwmx = sbas.v[VarType::MaxPosPowSub.tousz()].v;
    let maxbess = ac.v(SUB_BESS_MAX_MWH);

    /*
    let bess_solar = ac.v(SOLAR_TO_BESS_FACTOR);
    let sola = sbas.v[VarType::SolarEnergy as usize].v * bess_solar;
    let bess = if sola > maxbess { maxbess } else { sola };
    */

    if true {
        return (sub_save, svg_save, dif_save, eng_save, 0f32);
    }

    //let mut bess = 0f32;
    let bess;
    if let Some(rpf) = rpfm.get(&sbas.sbid) && rpf.rpf_avg>1.0 && rpf.rpf_fre>100 {
        /*
        println!(
            "!!!!! RPF sb:{} mwh:{} fre:{} max:{} avg:{} bess:{bess}",
            sbas.sbid, rpf.rpf_mwh, rpf.rpf_fre, rpf.rpf_max, rpf.rpf_avg
        );
        */
        bess = if rpf.rpf_avg>maxbess { maxbess } else { rpf.rpf_avg};
    } else {
        return (sub_save, svg_save, dif_save, eng_save, 0f32);
    };

    /*
    if pwmx <= 0.0 {
        return (sub_save, svg_save, dif_save, eng_save, 0f32);
    }
    */

    let ls_ex_en = bess;
    //println!("BESS: {bess} mx:{maxbess}");

    let trlm = sb.mvxn as f32 * ac.v(BC_POWER_FACT) * ac.v(BC_TR_LOAD_LIM);
    //let trcr = sb.mvxn as f32 * ac.v(BC_POWER_FACT) * ac.v(BC_TR_CRIT_LIM);
    //let dppy = trlm * grw / 100f32; // MW/yr increase
    //let yrno = (trlm - pwmx) / dppy;
    //let yrno = yrno as usize;
    //let mut ls_ex_en = 0f32;
    //let sola = sbas.v[VarType::SolarEnergy.tousz()].v;
    //let peek = -sbas.v[VarType::SubSolarPeekMw as usize].v / 1_000f32;
    /*
    let sort = sola / trlm;
    let daylp = if let Some(reps) = &sb.lp_rep_24.pos_rep.val {
        reps.iter().flatten().cloned().collect::<Vec<_>>()
    } else {
        vec![0f32; 96]
    };
    */
    if trlm <= 0.0 {
        return (sub_save, svg_save, dif_save, eng_save, 0f32);
    }

    //println!("LOG RAW trlm:{trlm} pwmx:{pwmx} grw:{grw}");
    let pwrt = log2_raw(trlm / pwmx);
    let grwl = log2_raw(1f32 + grw / 100f32);
    let yrnf = pwrt / grwl;
    let yrnn = yrnf as i32;
    let yyst = if yrnn < 4 { 0 } else { yrnn - 4 };

    let r = ac.v(BC_DISCN_RATE) / 100f32;
    let n = ac.v(BC_SUBST_YLEN);
    let anrt = (1f32 - Pow::pow(1f32 + r, -n)) / r;
    let sbcs = ac.v(BC_SUBST_COST);
    let ancs = sbcs / anrt;
    let ancs = ancs * 1_000_000f32;
    //let cst: Vec<f64> = vec![ancs.into(); 25];
    for y in 0..12 {
        let ya = if y < yyst { 0f32 } else { ancs };
        sub_save.push(ya);
    }

    let pkt = pwmx;
    let qbes = (pkt - trlm) * 0.4663; // tan 25
    let qbes = if qbes < 0f32 { 0f32 } else { qbes };
    //let qcst = qbes * 4f32; // 4 million bht
    let qcst = qbes * ac.v(BC_SVG_PERMW_COST); // 4 million bht
    let qcst = qcst / sbcs * ancs;
    //let qcst = qcst * 1_000_000f32;
    for _y in 0..12 {
        svg_save.push(qcst);
    }
    let yr_diff = bess * (ac.v(BC_ON_PEAK_COST) - ac.v(BC_OFFPEAK_COST)) * 1000f32;
    for _y in 0..12 {
        dif_save.push(yr_diff * ac.v(BC_NO_DAY_IN_YEAR));
    }

    let tm_pf: Vec<_> = if let Some(reps) = &sb.lp_rep_24.pos_rep.val {
        reps.iter().flatten().cloned().collect()
    } else {
        vec![0f32; 96]
    };
    let (p1, p2) = pow_calc_peak(&tm_pf, ac);
    let en_onp = p1.p_en;
    let en_ofp = p2.p_en;
    let enn = en_onp * Pow::pow(1f32 + grw / 100f32, yyst as f32);
    let enf = en_ofp * Pow::pow(1f32 + grw / 100f32, yyst as f32);
    let uc_onp = ac.v(BC_SELL_PRICE) - ac.v(BC_ON_PEAK_COST);
    let uc_ofp = ac.v(BC_SELL_PRICE) - ac.v(BC_OFFPEAK_COST);
    for y in 0..12 {
        let ya = if y < yyst {
            0f32
        } else {
            let aennx = en_onp * Pow::pow(1f32 + grw / 100f32, y as f32) - enn;
            let aenfx = en_ofp * Pow::pow(1f32 + grw / 100f32, y as f32) - enf;
            let aenny = aennx * uc_onp * 1000f32 * ac.v(BC_NO_DAY_IN_YEAR);
            let aenfy = aenfx * uc_ofp * 1000f32 * ac.v(BC_NO_DAY_IN_YEAR);
            (aenny + aenfy) * 0.94f32
        };
        eng_save.push(ya);
    }

    /*
    let o1 = sub_save.iter().sum::<f32>();
    let o2 = svg_save.iter().sum::<f32>();
    let o3 = dif_save.iter().sum::<f32>();
    let o4 = eng_save.iter().sum::<f32>();
    println!(
        "BEN_BESS #2 >>> sbcst:{} qcst:{} px:{} trlm:{} yrno:{} re:{} bes:{} ",
        ancs.pan(2),
        qcst.pan(2),
        pwmx.pan(0),
        trlm.pan(0),
        yrnn,
        peek.pan(0),
        bess.pan(0)
    );
    println!(
        "   sub:{} svg:{} dif:{} en:{}   {},{},{},{}",
        o1.pan(0),
        o2.pan(0),
        o3.pan(0),
        o4.pan(0),
        sub_save.len(),
        svg_save.len(),
        dif_save.len(),
        eng_save.len(),
    );
    */
    (sub_save, svg_save, dif_save, eng_save, ls_ex_en)
}

pub fn pow_calc_peak(time_v: &[f32], ac: &Assumption) -> (PowerCalc, PowerCalc) {
    let mut pwn = PowerCalc::default();
    let mut pwf = PowerCalc::default();
    for (i, v) in time_v.iter().enumerate() {
        if (ac.u(BC_ON_PEAK_BEGIN)..ac.u(BC_ON_PEAK_END)).contains(&i) {
            if *v >= 0f32 {
                pwn.p_sum += *v;
                pwn.p_cnt += 1;
                if *v > pwn.p_pk {
                    pwn.p_pk = *v;
                }
            } else {
                pwn.n_sum += -*v;
                pwn.n_cnt += 1;
                if -*v > pwn.n_pk {
                    pwn.n_pk = -*v;
                }
            }
        } else if *v >= 0f32 {
            pwf.p_sum += *v;
            pwf.p_cnt += 1;
            if *v > pwf.p_pk {
                pwf.p_pk = *v;
            }
        } else {
            pwf.n_sum += -*v;
            pwf.n_cnt += 1;
            if -*v > pwf.n_pk {
                pwf.n_pk = -*v;
            }
        }
    }
    pwn.p_en = pwn.p_sum / 2f32;
    pwn.n_en = pwn.n_sum / 2f32;
    if pwn.p_cnt > 0 {
        pwn.p_avg = pwn.p_sum / pwn.p_cnt as f32;
    }
    if pwn.n_cnt > 0 {
        pwn.n_avg = pwn.n_sum / pwn.n_cnt as f32;
    }
    pwf.p_en = pwf.p_sum / 2f32;
    pwf.n_en = pwf.n_sum / 2f32;
    if pwf.p_cnt > 0 {
        pwf.p_avg = pwf.p_sum / pwf.p_cnt as f32;
    }
    if pwf.n_cnt > 0 {
        pwf.n_avg = pwf.n_sum / pwf.n_cnt as f32;
    }
    (pwn, pwf)
}

///////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////
