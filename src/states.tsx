import {atom, AtomEffect, DefaultValue, selector, selectorFamily, useRecoilValue, useSetRecoilState} from "recoil";
import {PreConfig} from "./bindings/PreConfig";
import _ from "lodash";
import {PreRule} from "./bindings/PreRule";
import {equalSelector, equalSelectorFamily} from "./equalSelector";
import {PreDirectory} from "./bindings/PreDirectory";
import {ScanStatus} from "./bindings/ScanStatus";
import {ExclusionActionBatch} from "./bindings/ExclusionActionBatch";
import {useEffect} from "react";

const initialFetchConfig = async () => {
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

export const finalConfigState = atom({
  key: 'finalConfig',
  default: initialFetchConfig(),
  effects: [finalConfigEffect,]
})

export const draftConfigState = atom({
  key: 'draftConfig',
  default: finalConfigState,
})

export const noIncludeState = selector<boolean>({
  key: "noInclude",
  get: ({get}) => {
    const config = get(draftConfigState);
    return (config?.["no-include"]) ?? false;
  },
  set: ({set}, newValue) => {
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
    return (config?.rules) ?? {};
  },
  set: ({set}, newValue) => {
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
    return rules[ruleName];
  },
  set: (ruleName: string) => ({set}, newValue) => {
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
    return (config?.directories) ?? [];
  },
  set: ({set}, newValue) => {
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
    return dirs.find(dir => dir.path === dirPath)!;
  },
  set: (dirPath: string) => ({set}, newValue) => {
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
    return (config?.skips) ?? [];
  },
  set: ({set}, newValue) => {
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

const actionBatchEffect: AtomEffect<ExclusionActionBatch> = ({setSelf}) => {
  const f = async () => {
    if (typeof window === "undefined") {
      return () => {
      };
    }
    const listen = await import("@tauri-apps/api/event").then(tauri => tauri.listen);
    return await listen<ScanStatus>("scan_status_changed", ({payload}) => {
      if (payload.step === "result") {
        console.log("action batch", payload.content);
        setSelf(payload.content);
      }
    });
  }
  const unlisten = f();
  return () => {
    unlisten.then(unlisten => unlisten());
  }
};

const fetchActionBatch = async () => {
  const defaultValue = {
    add: [],
    remove: [],
  };
  if (typeof window === "undefined") {
    return defaultValue;
  }
  const invoke = await import("@tauri-apps/api").then(tauri => tauri.invoke);
  const scan_status = await invoke<ScanStatus>("scan_status");
  return scan_status.step === "result" ? scan_status.content : defaultValue;
};


export const actionBatchState = atom({
  key: 'actionBatch',
  default: fetchActionBatch(),
  effects: [actionBatchEffect,]
})

export const selectedActionBatchState = atom<ExclusionActionBatch>({
  key: 'selectedActionBatch',
  default: {
    remove: [],
    add: []
  },
})

export const selectedAddActionBatchState = selector({
  key: 'selectedAddActionBatch',
  get: ({get}) => {
    const selected = get(selectedActionBatchState);
    return selected.add;
  },
  set: ({set}, newValue) => {
    set(selectedActionBatchState, (prev) => ((!(newValue instanceof DefaultValue) && prev !== null) ? {
      ...prev,
      add: newValue
    } : prev));
  }
});

export const selectedRemoveActionBatchState = selector({
  key: 'selectedRemoveActionBatch',
  get: ({get}) => {
    const selected = get(selectedActionBatchState);
    return selected.remove;
  },
  set: ({set}, newValue) => {
    set(selectedActionBatchState, (prev) => ((!(newValue instanceof DefaultValue) && prev !== null) ? {
      ...prev,
      remove: newValue
    } : prev));
  }
});

export const SyncActionBatch = () => {
  const set = useSetRecoilState(selectedActionBatchState);
  const initial = useRecoilValue(actionBatchState);
  useEffect(() => {
    console.log("SyncActionBatch", initial);
    if (initial !== null) {
      set({
        add: [...initial.add],
        remove: [],
      });
    }
  }, [initial, set]);
  return null;
}

export const scanDetailState = atom({
  key: "scanDetail",
  default: false
});