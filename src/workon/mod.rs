use ansi_term::Style;
use config;
use config::Project;
use errors::AppError;
use slog::Logger;
use std::env;
use sync;
use ansi_term::Colour;


pub fn ls(maybe_config: Result<config::Config, AppError>) -> Result<(), AppError> {
  let config = maybe_config?;
  for (name, _) in config.projects {
    println!("{}", name)
  };
  Ok(())
}

pub fn print_path(maybe_config: Result<config::Config, AppError>, name: &str, logger: &Logger) -> Result<(), AppError> {
  let config = maybe_config?;
  let project = config.projects.get(name).ok_or_else(|| {
    AppError::UserError(format!("project {} not found", name))
  })?;
  let canonical_project_path = config.actual_path_to_project(project, logger);
  let path = canonical_project_path.to_str().ok_or(
    AppError::InternalError(
      "project path is not valid unicode",
    ),
  )?;
  println!("{}", path);
  Ok(())
}

pub fn inspect(name: &str, maybe_config: Result<config::Config, AppError>, logger: &Logger) -> Result<(), AppError> {
  let config = maybe_config?;
  let project = config.projects.get(name).ok_or_else(|| {
    AppError::UserError(format!("project {} not found", name))
  })?;
  println!(
    "{}",
    Style::new().underline().bold().paint(project.name.clone())
  );
  let project = config.projects.get(name).ok_or_else(|| {
    AppError::UserError(format!("project {} not found", name))
  })?;
  let canonical_project_path = config.actual_path_to_project(project, logger);
  let path = canonical_project_path.to_str().ok_or(
    AppError::InternalError(
      "project path is not valid unicode",
    ),
  )?;
  println!("{:<20}: {}", "Path", path);
  let tags = project.tags
                    .clone()
                    .map(|t| {
    let project_tags: Vec<String> = t.into_iter().collect();
    project_tags.join(", ")
  })
                    .unwrap_or_else(|| "None".to_owned());
  println!("{:<20}: {}", "Tags", tags);
  let git = project.git.clone();
  println!("{:<20}: {}", "Git", git);
  Ok(())
}

pub fn reworkon(maybe_config: Result<config::Config, AppError>, logger: &Logger) -> Result<(), AppError> {
  let config = maybe_config?;
  let os_current_dir = env::current_dir()?;
  let current_dir = os_current_dir.to_string_lossy().to_owned();
  let maybe_match = config.projects.values().find(|&p| config.actual_path_to_project(p, logger).to_string_lossy().eq(&current_dir));
  let project = maybe_match.ok_or_else(|| AppError::UserError(format!("No project matching expanded path {} found in config", current_dir)))?;
  let workon_cmd = format!("cd {} {}", current_dir, config.resolve_after_workon(logger, project));
  debug!(logger, "Reworkon match: {:?} with command {:?}", project, workon_cmd);
  let shell = sync::project_shell(&config.settings);
  sync::spawn_maybe(&shell, &workon_cmd, &os_current_dir, &project.name, &Colour::Yellow, logger)
}

pub fn gen(name: &str, maybe_config: Result<config::Config, AppError>, quick: bool, logger: &Logger) -> Result<(), AppError> {
  let config = maybe_config?;
  let project: &Project = config.projects.get(name).ok_or_else(|| {
    AppError::UserError(format!("project key {} not found in ~/.fw.json", name))
  })?;
  let canonical_project_path = config.actual_path_to_project(project, logger);
  let path = canonical_project_path.to_str().ok_or(
    AppError::InternalError(
      "project path is not valid unicode",
    ),
  )?;
  if !canonical_project_path.exists() {
    Err(AppError::UserError(format!(
      "project key {} found but path {} does not exist",
      name,
      path
    )))
  } else {
    let after_workon = if !quick {
      config.resolve_after_workon(logger, project)
    } else {
      String::new()
    };
    println!("cd {}{}", path, after_workon);
    Ok(())
  }
}
