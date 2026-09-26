import { invoke } from "@tauri-apps/api/core";
import type { Browser, Config, Rule, Simulation } from "./types";
export const api = {
  config: () => invoke<Config>("get_config"),
  platform: () => invoke<{ platform: string; canRegister: boolean; detail: string }>("get_platform_status"),
  defaults: () => invoke<void>("open_default_apps"),
  registerHandlers: () => invoke<void>("register_handlers"),
  detect: () => invoke<Browser[]>("detect_browsers"),
  saveRule: (rule: Rule) => invoke<void>("save_rule", { rule }),
  deleteRule: (id: string) => invoke<void>("delete_rule", { id }),
  simulate: (raw: string) => invoke<Simulation>("simulate_url", { raw }),
  conflicts: (rule: Rule) => invoke<Rule[]>("find_conflicts", { rule }),
  launchChoice: (raw: string, browserId: string, profile: string | undefined, privateWindow: boolean, remember: boolean) => invoke<void>("launch_choice", { raw, browserId, profile, private: privateWindow, remember }),
  export: (fullBackup = false) => invoke<string>("export_rules", { fullBackup })
};
