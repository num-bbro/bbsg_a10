use crate::asm::ASM::*;
use crate::utl6::Assumption;

pub fn cst_m1p_ins(no: f32, ac: &Assumption) -> Vec<f32> {
    let cst = ac.v(M1P_COST) * no / 3.0;
    let mut csts = Vec::<f32>::new();
    for _i in 0..3 {
        csts.push(cst);
    }
    csts
}

pub fn cst_m3p_ins(no: f32, ac: &Assumption) -> Vec<f32> {
    let cst = ac.v(M3P_COST) * no / 3.0;
    let mut csts = Vec::<f32>::new();
    for _i in 0..3 {
        csts.push(cst);
    }
    csts
}

pub fn cst_tr_ins(no: f32, ac: &Assumption) -> Vec<f32> {
    let cst = ac.v(TRX_COST) * no / 3.0;
    let mut csts = Vec::<f32>::new();
    for _i in 0..3 {
        csts.push(cst);
    }
    csts
}

pub fn cst_bes_ins(bescap: f32, ac: &Assumption) -> Vec<f32> {
    let cst = ac.v(ESS_COST) * bescap / 3.0;
    let mut csts = Vec::<f32>::new();
    for _i in 0..3 {
        csts.push(cst);
    }
    csts
}

pub fn cst_plfm_ins(no: f32, ac: &Assumption) -> Vec<f32> {
    let cst = ac.v(PLATFORM_COST) * no / 3.0;
    let mut csts = Vec::<f32>::new();
    for _i in 0..3 {
        csts.push(cst);
    }
    csts
}
pub fn cst_comm_ins(_bescap: f32, _ac: &Assumption) -> Vec<f32> {
    vec![0.0]
}

pub fn cst_m1p_imp(no: f32, ac: &Assumption) -> Vec<f32> {
    let cst = ac.v(M1P_IMP_COST) * no as f32 / 3.0;
    let mut csts = Vec::<f32>::new();
    for _i in 0..3 {
        csts.push(cst);
    }
    csts
}

pub fn cst_m3p_imp(no: f32, ac: &Assumption) -> Vec<f32> {
    let cst = ac.v(M3P_IMP_COST) * no as f32 / 3.0;
    let mut csts = Vec::<f32>::new();
    for _i in 0..3 {
        csts.push(cst);
    }
    csts
}

pub fn cst_tr_imp(no: f32, ac: &Assumption) -> Vec<f32> {
    let cst = ac.v(TRX_IMP_COST) * no / 3.0;
    let mut csts = Vec::<f32>::new();
    for _i in 0..3 {
        csts.push(cst);
    }
    csts
}

pub fn cst_bes_imp(_bescap: f32, _ac: &Assumption) -> Vec<f32> {
    vec![0.0]
}
pub fn cst_plfm_imp(_bescap: f32, _ac: &Assumption) -> Vec<f32> {
    vec![0.0]
}
pub fn cst_comm_imp(_bescap: f32, _ac: &Assumption) -> Vec<f32> {
    vec![0.0]
}

pub fn cst_reinvest(reinv: f32, _ac: &Assumption) -> Vec<f32> {
    let mut csts = vec![0.0, 0.0, 0.0];
    for _i in 0..12 {
        csts.push(reinv);
    }
    csts
}

pub fn cst_m1p_op(no: f32, ac: &Assumption) -> Vec<f32> {
    let mut cst = ac.v(M1P_OP_COST) * no;
    for _i in 0..3 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
    }
    let mut csts = vec![0.0, 0.0, 0.0];
    for _i in 0..12 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
        csts.push(cst);
    }
    csts
}

pub fn cst_m3p_op(no: f32, ac: &Assumption) -> Vec<f32> {
    let mut cst = ac.v(M3P_OP_COST) * no;
    for _i in 0..3 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
    }
    let mut csts = vec![0.0, 0.0, 0.0];
    for _i in 0..12 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
        csts.push(cst);
    }
    csts
}

pub fn cst_tr_op(no: f32, ac: &Assumption) -> Vec<f32> {
    let mut cst = ac.v(TRX_OP_COST) * no;
    let mut csts = vec![0.0, 0.0, 0.0];
    for _i in 0..3 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
    }
    for _i in 0..12 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
        csts.push(cst);
    }
    csts
}

pub fn cst_bes_op(bescap: f32, ac: &Assumption) -> Vec<f32> {
    let mut cst = bescap * ac.v(ESS_OP_COST) / 3.0;
    let mut csts = vec![0.0, 0.0, 0.0];
    for _i in 0..3 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
    }
    for _i in 0..12 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
        csts.push(cst);
    }
    csts
}

pub fn cst_plfm_op(no: f32, ac: &Assumption) -> Vec<f32> {
    let mut cst = ac.v(PLATFORM_OP_COST) * no;
    let mut csts = vec![0.0, 0.0, 0.0];
    for _i in 0..3 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
    }
    for _i in 0..12 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
        csts.push(cst);
    }
    csts
}

pub fn cst_comm_op(no: f32, ac: &Assumption) -> Vec<f32> {
    let mut cst = ac.v(COMM_OP_COST) * no * 12.0;
    for _i in 0..3 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
    }
    let mut csts = vec![0.0, 0.0, 0.0];
    for _i in 0..12 {
        cst *= 1.0 + ac.v(OP_INC_RATE);
        csts.push(cst);
    }
    csts
}

pub fn eir_cust_loss_save(no: f32, _ac: &Assumption) -> Vec<f32> {
    let cst = no * 0.01;
    let mut csts = Vec::<f32>::new();
    for _i in 0..15 {
        csts.push(cst);
    }
    csts
}
pub fn eir_cust_save(no: f32, _ac: &Assumption) -> Vec<f32> {
    let cst = no * 0.05;
    let mut csts = Vec::<f32>::new();
    for _i in 0..15 {
        csts.push(cst);
    }
    csts
}
pub fn eir_ghg_save(no: f32, _ac: &Assumption) -> Vec<f32> {
    let cst = no * 0.10;
    let mut csts = Vec::<f32>::new();
    for _i in 0..15 {
        csts.push(cst);
    }
    csts
}
pub fn eir_cust_mv_rev(no: f32, _ac: &Assumption) -> Vec<f32> {
    let cst = no * 0.13;
    let mut csts = Vec::<f32>::new();
    for _i in 0..15 {
        csts.push(cst);
    }
    csts
}
pub fn eir_cust_ev_save(no: f32, _ac: &Assumption) -> Vec<f32> {
    let cst = no * 0.09;
    let mut csts = Vec::<f32>::new();
    for _i in 0..15 {
        csts.push(cst);
    }
    csts
}
pub fn eir_cust_etruck_save(no: f32, _ac: &Assumption) -> Vec<f32> {
    let cst = no * 0.08;
    let mut csts = Vec::<f32>::new();
    for _i in 0..15 {
        csts.push(cst);
    }
    csts
}
pub fn eir_cust_solar_roof(no: f32, _ac: &Assumption) -> Vec<f32> {
    let cst = no * 0.06;
    let mut csts = Vec::<f32>::new();
    for _i in 0..15 {
        csts.push(cst);
    }
    csts
}
pub fn eir_en_rev_save(no: f32, _ac: &Assumption) -> Vec<f32> {
    let cst = no * 0.10;
    let mut csts = Vec::<f32>::new();
    for _i in 0..15 {
        csts.push(cst);
    }
    csts
}
