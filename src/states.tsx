import {atom, AtomEffect, DefaultValue, selector, useRecoilValue, useSetRecoilState} from "recoil";
import {PreConfig} from "./bindings/PreConfig";
import _ from "lodash";
import {PreRule} from "./bindings/PreRule";
import {equalSelector, equalSelectorFamily} from "./equalSelector";
import {PreDirectory} from "./bindings/PreDirectory";
import {ScanStatus} from "./bindings/ScanStatus";
import {ExclusionActionBatch} from "./bindings/ExclusionActionBatch";
import {useEffect} from "react";
import {ApplyErrors} from "./bindings/ApplyErrors";
import {
  disableAutoStart,
  enableAutoStart,
  getAutoStart,
  getConfig,
  getStore,
  scanStatus,
  setConfig,
  setStore
} from "./commands";
import i18n from "./i18n";

const finalConfigEffect: AtomEffect<PreConfig | null> = ({
                                                           setSelf,
                                                           onSet
                                                         }) => {
  onSet((newValue) => {
    if (newValue !== null) {
      setConfig(newValue);
    }
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
  default: getConfig(),
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
    const config = get(draftConfigState);
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
  set: (_dirPath: string) => ({set}, newValue) => {
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
  default: scanStatus(),
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

export const actionBatchState = equalSelector({
  key: "actionBatch",
  get: ({get}) => {
    const scanStatus = get(scanStatusState);
    if (scanStatus.step === "result") {
      return scanStatus.content;
    } else {
      return {
        add: [],
        remove: [],
      };
    }
  },
  equals: _.isEqual
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
    if (initial !== null) {
      set({
        add: [...initial.add],
        remove: [],
      });
    }
  }, [initial, set]);
  return null;
}

export type ScanPage = "scan" | "detail" | "applying" | "done" | "log";

export const scanPageState = atom<ScanPage>({
  key: "scanPage",
  default: "scan"
});

export const applyErrorsState = atom<ApplyErrors | null>({
  key: "applyErrors",
  default: null,
});

const autoStartEffect: AtomEffect<boolean> = ({onSet, setSelf}) => {
  onSet((newValue) => {
    const f = async () => {
      if (newValue) {
        await enableAutoStart();
      } else {
        await disableAutoStart();
      }
      setSelf(await getAutoStart());
    };
    f();
  });
};

export const autoStartState = atom<boolean>({
  key: "autoStart",
  default: getAutoStart(),
  effects: [autoStartEffect,]
})

const getLanguage = async () => await getStore("language") ?? "auto";

const languageEffect: AtomEffect<string> = ({onSet, setSelf}) => {
  const onChange = (language: string) => {
    setSelf(language);
    i18n.changeLanguage();
  }
  onSet((newValue) => {
    setStore("language", newValue);
  });

  const f = async () => {
    if (typeof window === "undefined") {
      return () => {
      };
    }
    const listen = await import("@tauri-apps/api/event").then(tauri => tauri.listen);
    return await listen<any>("properties_changed", ({payload}) => {
      if ("language" in payload) {
        onChange(payload.language);
      }
    });
  }
  const unlisten = f();
  return () => {
    unlisten.then(unlisten => unlisten());
  }
};

export const languageState = atom<string>({
  key: "language",
  default: getLanguage(),
  effects: [languageEffect,]
})