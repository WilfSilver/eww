use anyhow::{bail, Result};
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
    pub window_name: String,
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
            window_name: config_name,
            instance_id: id,
            pos: WindowArguments::extract_value_from_args::<Coords>("pos", &mut args)?,
            size: WindowArguments::extract_value_from_args::<Coords>("size", &mut args)?,
            monitor: WindowArguments::extract_value_from_args::<MonitorIdentifier>("screen", &mut args)?,
            anchor: WindowArguments::extract_value_from_args::<AnchorPoint>("anchor", &mut args)?,
            duration: WindowArguments::extract_value_from_args::<DynVal>("duration", &mut args)?
                .map(|x| x.as_duration())
                .transpose()?,
            args,
        };

        Ok(initiator)
    }

    pub fn extract_value_from_args<T: FromStr>(name: &str, args: &mut HashMap<VarName, DynVal>) -> Result<Option<T>, T::Err> {
        args.remove(&VarName(name.to_string())).map(|x| T::from_str(&x.0)).transpose()
    }

    /// Return a hashmap of all arguments the window was passed and expected, returning
    /// an error in case required arguments are missing or unexpected arguments are passed.
    pub fn get_local_window_variables(&self, window_def: &WindowDefinition) -> Result<HashMap<VarName, DynVal>> {
        let expected_args: HashSet<&String> = window_def.expected_args.iter().map(|x| &x.name.0).collect();
        let mut local_variables: HashMap<VarName, DynVal> = HashMap::new();

        // Ensure that the arguments passed to the window that are already interpreted by eww (id, screen)
        // are set to the correct values
        if expected_args.contains(&"id".to_string()) {
            local_variables.insert(VarName::from("id"), DynVal::from(self.instance_id.clone()));
        }
        if let Some(monitor) = &self.monitor && expected_args.contains(&"screen".to_string()) {
            local_variables.insert(VarName::from("screen"), DynVal::from(monitor));
        }

        local_variables.extend(self.args.clone());

        for attr in &window_def.expected_args {
            let name = VarName::from(attr.name.clone());
            if !local_variables.contains_key(&name) && !attr.optional {
                bail!("Error, missing argument '{}' when creating window with id '{}'", attr.name, self.instance_id);
            }
        }

        if local_variables.len() != window_def.expected_args.len() {
            let unexpected_vars: Vec<_> = local_variables.keys().cloned().filter(|n| !expected_args.contains(&n.0)).collect();
            bail!(
                "variables {} unexpectedly defined when creating window with id '{}'",
                unexpected_vars.join(", "),
                self.instance_id,
            );
        }

        Ok(local_variables)
    }
}
