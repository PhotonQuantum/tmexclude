import {atom, AtomEffect, DefaultValue, selector} from "recoil";
import {PreConfig} from "./bindings/PreConfig";
import _ from "lodash";
import {PreRule} from "./bindings/PreRule";
import {equalSelector, equalSelectorFamily} from "./equalSelector";
import {PreDirectory} from "./bindings/PreDirectory";
import {ScanStatus} from "./bindings/ScanStatus";

const initialFetchConfig = async () => {
  console.log("fetching config");
  if (typeof window === "undefined") {
    return null;
  }
  const invoke = await import("@tauri-apps/api").then(tauri => tauri.invoke);
  return await invoke<PreConfig>("get_config");
};

const finalConfigEffect: AtomEffect<PreConfig | null> = ({
                                                           setSelf,
                                                           onSet
                                                         }) => {
  onSet((newValue) => {
    if (typeof window === "undefined") {
      return;
    }
    const f = async () => {
      const invoke = await import("@tauri-apps/api").then(tauri => tauri.invoke);
      console.log("setting config");
      // TODO fix deadlock
      return await invoke("set_config", {config: newValue});
    }
    f();
  });

  const f = async () => {
    if (typeof window === "undefined") {
      return () => {
      };
    }
    const listen = await import("@tauri-apps/api/event").then(tauri => tauri.listen);
    return await listen<PreConfig>("config_changed", ({payload}) => {
      setSelf(payload);
    });
  }
  const unlisten = f();
  return () => {
    unlisten.then(unlisten => unlisten());
  }
};

// TODO separate draft and final config
export const finalConfigState = atom({
  key: 'finalConfig',
  default: initialFetchConfig(),
  effects: [finalConfigEffect,]
})

// TODO separate draft and final config
export const draftConfigState = atom({
  key: 'draftConfig',
  default: finalConfigState,
})

export const noIncludeState = selector<boolean>({
  key: "noInclude",
  get: ({get}) => {
    const config = get(draftConfigState);
    console.log("fetch config from noInclude", config);
    return (config?.["no-include"]) ?? false;
  },
  set: ({set}, newValue) => {
    console.log("set no-include", newValue);
    set(draftConfigState, (prev) => ((!(newValue instanceof DefaultValue) && prev !== null) ? {
      ...prev,
      "no-include": newValue
    } : prev));
  }
})

export const rulesState = selector<Record<string, PreRule>>({
  key: "rules",
  get: ({get}) => {
    const config = get(draftConfigState);
    console.log("fetch config from rules", config);
    return (config?.rules) ?? {};
  },
  set: ({set}, newValue) => {
    console.log("set rules", newValue);
    set(draftConfigState, (prev) => ((!(newValue instanceof DefaultValue) && prev !== null) ? {
      ...prev,
      rules: newValue
    } : prev));
  }
})

export const ruleNamesState = equalSelector<string[]>({
  key: "ruleNames",
  get: ({get}) => Object.keys(get(rulesState)).sort(),
  equals: _.isEqual
})

export const perRuleState = equalSelectorFamily({
  key: "perRule",
  get: (ruleName: string) => ({get}) => {
    const rules = get(rulesState);
    console.log("fetch rule", ruleName, rules);
    return rules[ruleName];
  },
  set: (ruleName: string) => ({set}, newValue) => {
    console.log("set rule", ruleName, newValue);
    set(rulesState, (prev) => ((!(newValue instanceof DefaultValue) && prev !== null) ? {
      ...prev,
      [ruleName]: newValue
    } : prev));
  },
  equals: _.isEqual
})

export const allPathsState = equalSelector({
  key: "allPaths",
  get: ({get}) => {
    const config = get(finalConfigState); // use final config here to avoid redundant re-render
    const rules = config?.rules ?? {};
    return _.sortedUniq(
      _.flatMap(Object.values(rules), rule => Array.isArray(rule) ? [] : [...rule.excludes, ...rule["if-exists"]])
        .sort());
  },
  equals: _.isEqual
})

export const dirsState = selector<PreDirectory[]>({
  key: "dirs",
  get: ({get}) => {
    const config = get(draftConfigState);
    console.log("fetch config from dirs", config);
    return (config?.directories) ?? [];
  },
  set: ({set}, newValue) => {
    console.log("set dirs", newValue);
    set(draftConfigState, (prev) => ((!(newValue instanceof DefaultValue) && prev !== null) ? {
      ...prev,
      directories: newValue
    } : prev));
  }
})

export const perDirState = equalSelectorFamily<PreDirectory, string>({
  key: "perDir",
  get: (dirPath: string) => ({get}) => {
    const dirs = get(dirsState);
    console.log("fetch dir", dirPath, dirs);
    return dirs.find(dir => dir.path === dirPath)!;
  },
  set: (dirPath: string) => ({set}, newValue) => {
    console.log("set dir", dirPath, newValue);
    set(dirsState, (prev) => ((!(newValue instanceof DefaultValue) && prev !== null) ?
      prev.map((dir) => dir.path === newValue.path ? newValue : dir) :
      prev));
  },
  equals: _.isEqual
});

export const skipsState = selector<string[]>({
  key: "skips",
  get: ({get}) => {
    const config = get(draftConfigState);
    console.log("fetch config from skips", config);
    return (config?.skips) ?? [];
  },
  set: ({set}, newValue) => {
    console.log("set skips", newValue);
    set(draftConfigState, (prev) => ((!(newValue instanceof DefaultValue) && prev !== null) ? {
      ...prev,
      skips: newValue
    } : prev));
  }
});

export const configChangedState = selector({
  key: "configChanged",
  get: ({get}) => {
    const draft = get(draftConfigState);
    const final = get(finalConfigState);
    return !_.isEqual(draft, final);
  }
});

const initialFetchScanStatus = async () => {
  if (typeof window === "undefined") {
    return {step: "idle"} as ScanStatus;
  }
  const invoke = await import("@tauri-apps/api").then(tauri => tauri.invoke);
  return await invoke<ScanStatus>("scan_status");
}

const scanStatusEffect: AtomEffect<ScanStatus> = ({setSelf}) => {
  const f = async () => {
    if (typeof window === "undefined") {
      return () => {
      };
    }
    const listen = await import("@tauri-apps/api/event").then(tauri => tauri.listen);
    return await listen<ScanStatus>("scan_status_changed", ({payload}) => {
      setSelf(payload);
    });
  }
  const unlisten = f();
  return () => {
    unlisten.then(unlisten => unlisten());
  }
}

export const scanStatusState = atom<ScanStatus>({
  key: "scanStatus",
  default: initialFetchScanStatus(),
  effects: [scanStatusEffect,]
})

export const scanStepState = selector({
  key: "scanStep",
  get: ({get}) => (get(scanStatusState).step),
})

export const scanCurrentState = selector({
  key: "scanCurrent",
  get: ({get}) => {
    const scanStatus = get(scanStatusState);
    if (scanStatus.step === "scanning") {
      return {
        path: scanStatus.content.current_path,
        found: scanStatus.content.found
      };
    } else {
      return {
        path: "N/A",
        found: 0
      };
    }
  }
})