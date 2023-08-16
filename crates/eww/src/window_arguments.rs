use anyhow::{anyhow, Result};
use eww_shared_util::VarName;
use simplexpr::dynval::DynVal;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use yuck::{
    config::{monitor::MonitorIdentifier, window_definition::WindowDefinition, window_geometry::AnchorPoint},
    value::Coords,
};

/// This stores the arguments given in the command line to create a window
/// While creating a window, we combine this with information from the
/// [`WindowDefinition`] to create a [WindowInitiator](`crate::window_initiator::WindowInitiator`), which stores all the
/// information required to start a window
#[derive(Debug, Clone)]
pub struct WindowArguments {
    /// Name of the window as defined in the eww config
    pub config_name: String,
    /// Instance ID of the window
    pub instance_id: String,
    pub anchor: Option<AnchorPoint>,
    pub args: HashMap<VarName, DynVal>,
    pub duration: Option<std::time::Duration>,
    pub monitor: Option<MonitorIdentifier>,
    pub pos: Option<Coords>,
    pub size: Option<Coords>,
}

impl WindowArguments {
    pub fn new_from_args(id: String, config_name: String, mut args: HashMap<VarName, DynVal>) -> Result<Self> {
        let initiator = WindowArguments {
            config_name,
            instance_id: id,
            pos: WindowArguments::extract_value_from_args::<Coords>("pos", &mut args)?,
            size: WindowArguments::extract_value_from_args::<Coords>("size", &mut args)?,
            monitor: WindowArguments::extract_value_from_args::<MonitorIdentifier>("screen", &mut args)?,
            anchor: WindowArguments::extract_value_from_args::<AnchorPoint>("anchor", &mut args)?,
            duration: WindowArguments::extract_value_from_args::<std::time::Duration>("duration", &mut args)?,
            args,
        };

        Ok(initiator)
    }

    pub fn extract_value_from_args<T: FromStr>(name: &str, args: &mut HashMap<VarName, DynVal>) -> Result<Option<T>, T::Err> {
        args.remove(&VarName(name.to_string())).map(|x| T::from_str(&x.0)).transpose()
    }

    pub fn get_local_window_variables(&self, window_def: &WindowDefinition) -> Result<HashMap<VarName, DynVal>> {
        let expected_args: HashSet<&String> = window_def.expected_args.iter().map(|x| &x.name.0).collect();
        let mut local_variables: HashMap<VarName, DynVal> = HashMap::new();

        // Inserts these first so they can be overridden
        if expected_args.contains(&"id".to_string()) {
            local_variables.insert(VarName::from("id"), DynVal::from(self.instance_id.clone()));
        }
        if self.monitor.is_some() && expected_args.contains(&"screen".to_string()) {
            let mon_dyn = match self.monitor.clone().unwrap() {
                MonitorIdentifier::Numeric(x) => DynVal::from(x),
                MonitorIdentifier::Name(x) => DynVal::from(x),
            };
            local_variables.insert(VarName::from("screen"), mon_dyn);
        }

        local_variables.extend(self.args.clone().into_iter());

        for attr in &window_def.expected_args {
            let name = VarName::from(attr.name.clone());

            // This is here to get around the map_entry warning
            let mut inserted = false;
            local_variables.entry(name).or_insert_with(|| {
                inserted = true;
                DynVal::from_string(String::new())
            });

            if inserted && !attr.optional {
                return Err(anyhow!("Error, {} was required when creating {} but was not given", attr.name, self.config_name));
            }
        }

        if local_variables.len() != window_def.expected_args.len() {
            let unexpected_vars: Vec<VarName> = local_variables
                .iter()
                .filter_map(|(n, _)| if !expected_args.contains(&n.0) { Some(n.clone()) } else { None })
                .collect();
            return Err(anyhow!(
                "'{}' {} unexpectedly defined when creating window {}",
                unexpected_vars.join(","),
                if unexpected_vars.len() == 1 { "was" } else { "were" },
                self.config_name
            ));
        }

        Ok(local_variables)
    }
}
