use std::process::exit;
use std::fs::read;
use std::env::var;
use colored::Colorize;
use inkwell::passes::*;
use inkwell::OptimizationLevel::*;
use inkwell::module::Module;
pub enum AdditionalArg {
    Null,
    Bool(bool),
    Int(i32),
}
use AdditionalArg::*;
pub fn add_pass(pm: &PassManager<Module>, name: &str, arg: AdditionalArg) -> bool { // I wish there was a better way to do this
    let name = name
        .replace("attr", "attribute")
        .replace("fn", "function")
        .replace("func", "function")
        .replace("argument", "arg")
        .replace("constant", "const")
        .replace("optimizer", "opt")
        .replace("optimization", "opt")
        .replace("alignment", "align")
        .replace("variable", "var")
        .replace("instruction", "inst")
        .replace("_", " ")
        .replace("-", " ")
        .to_lowercase();
    match name.as_str() {
        "arg promotion" => pm.add_argument_promotion_pass(),
        "const merge" => pm.add_constant_merge_pass(),
        "merge function" | "merge functions" => pm.add_merge_functions_pass(),
        "dead arg" | "dead args" | "dead arg elimination" | "dead args elimination" => pm.add_dead_arg_elimination_pass(),
        "function attr" | "function attrs" => pm.add_function_attrs_pass(),
        "function inline" | "function inlining" => pm.add_function_inlining_pass(),
        "always inline" | "always inliner" => pm.add_always_inliner_pass(),
        "global dce" => pm.add_global_dce_pass(),
        "global opt" => pm.add_global_optimizer_pass(),
        "prune eh" => pm.add_prune_eh_pass(),
        "ipsccp" => pm.add_ipsccp_pass(),
        "internalize" => pm.add_internalize_pass(match arg {Null => false, Bool(val) => val, Int(val) => val != 0}),
        "strip dead prototype" | "strip dead prototypes" | "strip prototype" | "strip prototypes" => pm.add_strip_dead_prototypes_pass(),
        "strip symbol" | "strip symbols" => pm.add_strip_symbol_pass(),
        "loop vectorize" => pm.add_loop_vectorize_pass(),
        "slp vectorize" => pm.add_slp_vectorize_pass(),
        "aggresive dce" | "adce" => pm.add_aggressive_dce_pass(),
        "bit tracking dce" | "bit dce" | "tracking dce" => pm.add_bit_tracking_dce_pass(),
        "align from assumptions" => pm.add_alignment_from_assumptions_pass(),
        "cfg simplification" => pm.add_cfg_simplification_pass(),
        "dead store" | "dead store elimination" => pm.add_dead_store_elimination_pass(),
        "scalarizer" => pm.add_scalarizer_pass(),
        "mlsm" | "merged load store motion" => pm.add_merged_load_store_motion_pass(),
        "gvn" | "global value numbering" => pm.add_gvn_pass(),
        "new gvn" | "new global value numbering" => pm.add_new_gvn_pass(),
        "ind var simplify" | "induction var simplify" => pm.add_ind_var_simplify_pass(),
        "inst combine" | "inst combining" => pm.add_instruction_combining_pass(),
        "jump thread" | "jump threading" => pm.add_jump_threading_pass(),
        "licm" => pm.add_licm_pass(),
        "dld" | "loop deletion" | "dead loop deletion" => pm.add_loop_deletion_pass(),
        "loop idiom" => pm.add_loop_idiom_pass(),
        "loop rotate" => pm.add_loop_rotate_pass(),
        "loop reroll" => pm.add_loop_reroll_pass(),
        "loop unroll" => pm.add_loop_unroll_pass(),
        "loop unswitch" => pm.add_loop_unswitch_pass(),
        "memcpy" | "memcpy opt" => pm.add_memcpy_optimize_pass(),
        "partially inline lib calls" => pm.add_partially_inline_lib_calls_pass(),
        "lower switch" => pm.add_lower_switch_pass(),
        "promote memory to register" => pm.add_promote_memory_to_register_pass(),
        "reassociate" => pm.add_reassociate_pass(),
        "sccp" => pm.add_sccp_pass(),
        "scalar repl aggregates" => match arg {
            Null | Bool(false) => pm.add_scalar_repl_aggregates_pass(),
            Bool(true) => pm.add_scalar_repl_aggregates_pass_ssa(),
            Int(val) => pm.add_scalar_repl_aggregates_pass_with_threshold(val)
        },
        "scalar repl aggregates ssa" => pm.add_scalar_repl_aggregates_pass_ssa(),
        "simplify lib calls" => pm.add_simplify_lib_calls_pass(),
        "tail call elimination" => pm.add_tail_call_elimination_pass(),
        "instruction simplify" => pm.add_instruction_simplify_pass(),
        "demote memory to register" => pm.add_demote_memory_to_register_pass(),
        "verify" | "verifier" => pm.add_verifier_pass(),
        "correlated value propagation" => pm.add_correlated_value_propagation_pass(),
        "early cse" => pm.add_early_cse_pass(),
        "early cse mem ssa" => pm.add_early_cse_mem_ssa_pass(),
        "lower expect intrinsic" => pm.add_lower_expect_intrinsic_pass(),
        "type based alias analysis" => pm.add_type_based_alias_analysis_pass(),
        "scoped no alias aa" => pm.add_scoped_no_alias_aa_pass(),
        "basic alias analysis" => pm.add_basic_alias_analysis_pass(),
        "aggressive inst combine" | "aggressive inst combiner" => pm.add_aggressive_inst_combiner_pass(),
        "loop unroll and jam" | "loop unroll jam" => pm.add_loop_unroll_and_jam_pass(),
        "coroutine early" => pm.add_coroutine_early_pass(),
        "coroutine split" => pm.add_coroutine_split_pass(),
        "coroutine elide" => pm.add_coroutine_elide_pass(),
        "coroutine cleanup" => pm.add_coroutine_cleanup_pass(),
        "coroutine" | "coroutines" => {
            pm.add_coroutine_early_pass();
            pm.add_coroutine_split_pass();
            pm.add_coroutine_elide_pass();
            pm.add_coroutine_cleanup_pass();
        },
        _ => return false
    }
    true
}
#[allow(non_snake_case)]
pub fn from_file(data: &str, pm: &PassManager<Module>) {
    let WARNING = "warning".bright_yellow().bold();
    for (n, mut line) in data.split("\n").enumerate() {
        let n = n + 1;
        if let Some(idx) = line.find('#') {line = &line[..idx];}
        if line.trim().len() == 0 {continue}
        if let Some(idx) = line.find('=') {
            let pass = line[..idx].trim();
            let val = line[(idx + 1)..].trim().to_lowercase();
            if val.len() == 0 {
                if !add_pass(pm, pass, Null) {
                    eprintln!("{WARNING}:{n}: unknown pass '{pass}'");
                }
            }
            else if val == "true" {
                if !add_pass(pm, pass, Bool(true)) {
                    eprintln!("{WARNING}:{n}: unknown pass '{pass}'");
                }
            }
            else if val == "false" {
                if !add_pass(pm, pass, Bool(false)) {
                    eprintln!("{WARNING}:{n}: unknown pass '{pass}'");
                }
            }
            else if let Ok(val) = val.parse() {
                if !add_pass(pm, pass, Int(val)) {
                    eprintln!("{WARNING}:{n}: unknown pass '{pass}'");
                }
            }
            else {
                eprintln!("{WARNING}:{n}: unrecognized value '{val}'");
                if !add_pass(pm, pass, Null) {
                    eprintln!("{WARNING}:{n}: unknown pass '{pass}'");
                }
            }
        }
        else {
            let pass = line.trim();
            if !add_pass(pm, pass, Null) {
                eprintln!("{WARNING}:{n}: unknown pass '{pass}'");
            }
        }
    }
}
pub fn load_profile(name: Option<&str>, pm: &PassManager<Module>) {
    let name = name.unwrap_or("default");
    if let Ok(cobalt_dir) = var("COBALT_DIR") {
        if let Ok(data) = read(format!("{cobalt_dir}/profiles/{name}")) {
            from_file(&String::from_utf8_lossy(data.as_slice()), pm);
            return;
        }
    }
    if let Ok(home_dir) = var("HOME") {
        if let Ok(data) = read(format!("{home_dir}/.cobalt/profiles/{name}")) {
            from_file(&String::from_utf8_lossy(data.as_slice()), pm);
            return;
        }
        if let Ok(data) = read(format!("{home_dir}/.config/cobalt/profiles/{name}")) {
            from_file(&String::from_utf8_lossy(data.as_slice()), pm);
            return;
        }
    }
    if let Ok(data) = read(format!("/usr/local/share/cobalt/profiles/{name}")) {
        from_file(&String::from_utf8_lossy(data.as_slice()), pm);
        return;
    }
    if let Ok(data) = read(format!("/usr/share/cobalt/profiles/{name}")) {
        from_file(&String::from_utf8_lossy(data.as_slice()), pm);
        return;
    }
    match name {
        "none" => {},
        "less" => {
            let pmb = PassManagerBuilder::create();
            pmb.set_optimization_level(Less);
            pmb.populate_module_pass_manager(&pm);
        },
        "default" => {
            let pmb = PassManagerBuilder::create();
            pmb.set_optimization_level(Default);
            pmb.populate_module_pass_manager(&pm);
        },
        "aggressive" => {
            let pmb = PassManagerBuilder::create();
            pmb.set_optimization_level(Aggressive);
            pmb.populate_module_pass_manager(&pm);
        },
        _ => {
            eprintln!("{}: couldn't find profile {name}", "error".bright_red().bold());
            exit(103);
        }
    }
}
