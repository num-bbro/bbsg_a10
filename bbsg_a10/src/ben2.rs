use crate::asm::ASM::*;
use crate::dcl::PeaAssVar;
use crate::dcl::VarType;
use crate::utl4::Archi;
use num::pow::Pow;
use sglib03::prc4::SubYearBenInfo;

use sglib03::prc2::PowerCalc;

pub fn ben_bill_accu(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_cash_flow(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_dr_save(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_boxline_save(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_work_save(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_sell_meter(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_emeter(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_mt_read(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_mt_disconn(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_tou_sell(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_tou_read(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_tou_update(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_outage_labor(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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
pub fn ben_reduce_complain(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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
pub fn ben_asset_value(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

pub fn ben_model_entry(tras: &PeaAssVar, ac: &Archi) -> Vec<f32> {
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

use crate::dcl::PeaSub;

pub fn ben_bess_calc(
    sb: &PeaSub,
    sbas: &PeaAssVar,
    ac: &Archi,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, f32) {
    /*
    let mut sub_save = Vec::<f32>::new();
    let mut svg_save = Vec::<f32>::new();
    let mut dif_save = Vec::<f32>::new();
    let mut eng_save = Vec::<f32>::new();
    */
    let mut sub_save = vec![0.0, 0.0, 0.0];
    let mut svg_save = vec![0.0, 0.0, 0.0];
    let mut dif_save = vec![0.0, 0.0, 0.0];
    let mut eng_save = vec![0.0, 0.0, 0.0];
    // ==============================================
    // ==============================================
    // ======= BEGIN =======
    //let grw = gr;
    //let pwmx = pwx;
    let grw = sbas.v[VarType::EnGrowth.tousz()].v;
    let pwmx = sbas.v[VarType::MaxPosPowSub.tousz()].v;

    let trlm = sb.mvxn as f32 * ac.v(BC_POWER_FACT) * ac.v(BC_TR_LOAD_LIM);
    let trcr = sb.mvxn as f32 * ac.v(BC_POWER_FACT) * ac.v(BC_TR_CRIT_LIM);
    let dppy = trlm * grw / 100f32; // MW/yr increase
    let yrno = (trlm - pwmx) / dppy;
    let yrno = yrno as usize;
    let mut ls_ex_en = 0f32;
    let sola = sbas.v[VarType::SolarEnergy.tousz()].v;
    let sort = sola / trlm;

    //println!("   >>>>>> SOLAR ENERGY {sola}");

    /*
    if sola > 0f32 {
        println!(
            "BEN_BESS 1 >>>> {sola} ==== sbtp:{} sort:{sort} yrno:{yrno}",
            sb.sbtp
        );
    }
    */

    //if sb.sbtp == "AIS" && yrno < BESS_YEAR_TO_FULL as usize && sort > BESS_SOLA_RATIO {
    //if sb.sbtp == "AIS" && sort > BESS_SOLA_RATIO && yrno < BESS_YEAR_TO_FULL as usize {
    //if sb.sbtp == "AIS" && sort > BESS_SOLA_RATIO {
    if ["GIS", "AIS"].contains(&sb.sbtp.as_str()) && sort > ac.v(BESS_SOLA_RATIO) {
        //println!("BEN_BESS 2 >>>>>>> {sola} =============");

        // day load profile
        let daylp = if let Some(reps) = &sb.lp_rep_24.pos_rep.val {
            reps.iter().flatten().cloned().collect::<Vec<_>>()
        } else {
            vec![0f32; 96]
        };

        use sglib03::prc4::BC_PROJ_YLEN;
        let mut yr_daypf = Vec::<Vec<f32>>::new();
        for i in 0..=BC_PROJ_YLEN {
            let mut day_prof = daylp.clone();
            for vapf in day_prof.iter_mut() {
                *vapf *= Pow::pow(1f32 + grw / 100f32, i as f32);
            }
            yr_daypf.push(day_prof);
        }

        let yr_start = yrno;
        let _mxrt = pwmx / trlm * 100f32;

        let r = ac.v(BC_DISCN_RATE) / 100f32;
        let n = ac.v(BC_SUBST_YLEN);
        let anrt = (1f32 - Pow::pow(1f32 + r, -n)) / r;
        let ancs = ac.v(BC_SUBST_COST) / anrt;
        let cst: Vec<f64> = vec![ancs.into(); 25];
        let mut subcst = Vec::<SubYearBenInfo>::new();
        for (i, v) in cst.iter().enumerate() {
            let fa = v / Pow::pow(1f64 + r as f64, i as f64);
            let be = if i < 15 {
                v * Pow::pow(1.03f64, i as f64)
            } else {
                0f64
            };
            subcst.push(SubYearBenInfo {
                year: i,
                sub_cost: *v as f32,
                sub_npv: fa as f32,
                sub_save: be as f32,
                ..Default::default()
            });
        }

        //let mut sbsav = 0f32;
        let be_start = if yr_start < 4 { 1 } else { yr_start - 3 };
        for _i in 1..be_start {
            sub_save.push(0f32);
        }
        for cst in subcst.iter().take(ac.u(BC_BESS_YLEN)).skip(be_start - 1) {
            //sbsav += cst.sub_save;
            sub_save.push(cst.sub_save * 1_000_000f32);
        }

        // power and energy of the last year
        let mut _ls_ex_sm = 0f32;
        let mut ls_ex_pw = 0f32;
        for tm_pf in yr_daypf[BC_PROJ_YLEN].iter() {
            let dv = tm_pf - trcr;
            if dv >= 0f32 {
                ls_ex_pw = dv.max(ls_ex_pw);
                _ls_ex_sm += dv;
            }
        }
        //ls_ex_en = ls_ex_sm * 0.25f32;

        ls_ex_en = sola * ac.v(BESS_CAP_SOLAR_RATIO);
        if ls_ex_en > ac.v(BESS_CAP_PILOT_MAX) {
            ls_ex_en = ac.v(BESS_CAP_PILOT_MAX);
        }
        //println!("BEN_BESS 3 >>>>>>> {ls_ex_en} =============");

        // load profile of year 2024
        let tm_pf: Vec<_> = if let Some(reps) = &sb.lp_rep_24.pos_rep.val {
            reps.iter().flatten().cloned().collect()
        } else {
            vec![0f32; 96]
        };
        let (p1, p2) = pow_calc_peak(&tm_pf, ac);
        // energy, onpeak, offpeak
        let p_en: f32 = tm_pf.iter().sum();
        let en_onp = p1.p_en;
        let en_ofp = p2.p_en;

        let grw = 2f32;
        let en0 = p_en * Pow::pow(1f32 + grw / 100f32, yr_start as f32);
        let enn = en_onp * Pow::pow(1f32 + grw / 100f32, yr_start as f32);
        let enf = en_ofp * Pow::pow(1f32 + grw / 100f32, yr_start as f32);

        let mut be_en_added = Vec::<f32>::new();
        for _n in 3..=yr_start {
            //print!(" {_n}");
            be_en_added.push(0f32);
            eng_save.push(0f32);
        }
        let _l1 = eng_save.len();

        let (mut _aen0, mut _aenn, mut _aenf) = (0f32, 0f32, 0f32);
        let uc_onp = ac.v(BC_SELL_PRICE) - ac.v(BC_ON_PEAK_COST);
        let uc_ofp = ac.v(BC_SELL_PRICE) - ac.v(BC_OFFPEAK_COST);

        //print!("   ");
        let yr0 = if yr_start == 0 { 2 } else { yr_start };
        let yr0 = if yr_start == 1 { 2 } else { yr0 };
        for n in yr0 + 1..BC_PROJ_YLEN {
            //print!(" {n}");
            let aennx = en_onp * Pow::pow(1f32 + grw / 100f32, n as f32) - enn;
            let aenfx = en_ofp * Pow::pow(1f32 + grw / 100f32, n as f32) - enf;

            let aenny = aennx * uc_onp * 1000f32 * ac.v(BC_NO_DAY_IN_YEAR);
            let aenfy = aenfx * uc_ofp * 1000f32 * ac.v(BC_NO_DAY_IN_YEAR);

            let aen = (aenny + aenfy) * 0.94f32;
            be_en_added.push(aen);
            eng_save.push(aen);
        }
        //println!();
        let _l2 = eng_save.len();
        /*
        if l2 > 15 {
            println!(
                "=== sub: {}  yr_start: {yr_start}, yr0:{yr0} LEN:{BC_PROJ_YLEN} len:{l1}->{l2}",
                sb.sbid,
            );
        }
        */

        let _en0 = en0 * ac.v(BC_NO_DAY_IN_YEAR);
        let _enn = enn * ac.v(BC_NO_DAY_IN_YEAR);
        let _enf = enf * ac.v(BC_NO_DAY_IN_YEAR);
        let ex_ben_onp = _aenn * uc_onp * 1000f32 * ac.v(BC_NO_DAY_IN_YEAR);
        let ex_ben_ofp = _aenf * uc_ofp * 1000f32 * ac.v(BC_NO_DAY_IN_YEAR);
        let _ex_ben = (ex_ben_onp + ex_ben_ofp) * 0.94f32;

        //let mut be_re_diff = Vec::<(u32, f32)>::new();
        let mut yr_diff = ls_ex_en * (ac.v(BC_ON_PEAK_COST) - ac.v(BC_OFFPEAK_COST)) * 1000f32;
        for _yi in 0..ac.u(BC_BESS_YLEN) {
            yr_diff *= 1.04;
            //be_re_diff.push((yi as u32 + 2028, yr_diff));
            dif_save.push(yr_diff * ac.v(BC_NO_DAY_IN_YEAR));
            //dif_save.push(55f32);
        }

        let _dec_ben = ls_ex_en
            * (ac.v(BC_ON_PEAK_COST) - ac.v(BC_OFFPEAK_COST))
            * 1000f32
            * ac.v(BC_NO_DAY_IN_YEAR);

        //let mut pkt = trcr;
        let pkt = pwmx;
        let qbes = (pkt - trlm) * 0.4663; // tan 25
        let qbes = if qbes < 0f32 { 0f32 } else { qbes };
        let qcst = qbes * 4f32; // 4 million bht
                                //println!("{qbes}- {qcst}");
        let _r = ac.v(BC_DISCN_RATE) / 100f32;
        //let fa = qcsy / Pow::pow(1f32 + r, 10f32);
        //
        let mut be_svg_save = Vec::<f32>::new();
        for _y in 3..=yr_start {
            be_svg_save.push(0f32);
            svg_save.push(0f32);
        }
        let l1 = svg_save.len();
        let mut _ben3 = 0f32;
        let yr1 = if l1 == 3 { 2 } else { yr_start };
        for n in yr1 + 1..BC_PROJ_YLEN {
            //for y in yr_start..=BC_PROJ_YLEN - 3 {
            //for y in yr_start..=BC_PROJ_YLEN - 3 {
            let be = qcst / 10f32 * Pow::pow(1.03f32, n as f32);
            be_svg_save.push(be * 1_000_000f32);
            _ben3 += be;
            svg_save.push(be * 1_000_000f32);
        }
        let _l2 = svg_save.len();
        //print!("SVG qbes:{qbes} pkt:{pkt} trlm:{trlm} qcst:{qcst} l1:{l1} l2:{l2}");

        //use sglib03::prc4::BC_PEA_PROFIT;

        let mut _ac_ex_sm = 0f32;
        let mut _ac_ex_be = 0f32;
        for yr_daypf in yr_daypf.iter() {
            for (i, tm_pf) in yr_daypf.iter().enumerate() {
                let dv = tm_pf - trlm;
                if dv >= 0f32 {
                    let up = if (ac.u(BC_ON_PEAK_BEGIN)..ac.u(BC_ON_PEAK_END)).contains(&i) {
                        let df = ac.v(BC_ON_PEAK_COST) - ac.v(BC_OFFPEAK_COST);
                        df + ac.v(BC_PEA_PROFIT)
                    } else {
                        ac.v(BC_PEA_PROFIT)
                    };
                    _ac_ex_sm += dv;
                    _ac_ex_be += dv * up * 0.5f32;
                }
            }
        }
        _ac_ex_sm *= ac.v(BC_NO_DAY_IN_YEAR);
        _ac_ex_be *= ac.v(BC_NO_DAY_IN_YEAR);
        //println!("BEN_BESS 4 >>>>>>> {ls_ex_en} =============");
        //println!( "====  trlm:{trlm} trcr:{trcr} pwmx:{pwmx} yrno:{yrno} grw:{grw} mxrt:{mxrt} sbsav:{sbsav}");
    }
    // ======= END =======
    // ==============================================
    // ==============================================
    (sub_save, svg_save, dif_save, eng_save, ls_ex_en)
}

pub fn pow_calc_peak(time_v: &[f32], ac: &Archi) -> (PowerCalc, PowerCalc) {
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
