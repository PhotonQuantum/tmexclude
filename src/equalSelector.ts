import {
  ReadOnlySelectorFamilyOptions,
  ReadOnlySelectorOptions,
  ReadWriteSelectorFamilyOptions,
  ReadWriteSelectorOptions,
  RecoilState,
  RecoilValueReadOnly,
  selector,
  selectorFamily,
  SerializableParam
} from "recoil";

interface EqualSelectorOptions<T> extends Pick<ReadOnlySelectorOptions<T>, "key" | "get">,
  Partial<Pick<ReadWriteSelectorOptions<T>, "set">> {
  equals: (a: T, b: T) => boolean;
}

export function equalSelector<T>(options: EqualSelectorOptions<T>): RecoilState<T> | RecoilValueReadOnly<T> {
  const inner = selector({
    key: `${options.key}_inner`,
    get: options.get
  });

  let prior: T | undefined;

  return selector({
    key: options.key,
    get: ({get}) => {
      const latest = get(inner);
      if (prior != null && options.equals(latest, prior)) {
        return prior;
      }
      console.log("equalSelector rerender", options.key, latest, prior);
      prior = latest;
      return latest as T;
    }, ...options.set ? {set: options.set} : {}
  });
}

interface ReadWriteEqualSelectorFamilyOptions<T, P extends SerializableParam>
  extends Pick<ReadWriteSelectorFamilyOptions<T, P>, "key" | "get" | "set"> {
  equals: (a: T, b: T) => boolean;
}

interface ReadOnlyEqualSelectorFamilyOptions<T, P extends SerializableParam>
  extends Pick<ReadOnlySelectorFamilyOptions<T, P>, "key" | "get"> {
  equals: (a: T, b: T) => boolean;
}

export function equalSelectorFamily<T, P extends SerializableParam>(options: ReadWriteEqualSelectorFamilyOptions<T, P>): (param: P) => RecoilState<T>;
export function equalSelectorFamily<T, P extends SerializableParam>(options: ReadOnlyEqualSelectorFamilyOptions<T, P>): (param: P) => RecoilValueReadOnly<T>;

export function equalSelectorFamily<T, P extends SerializableParam>(options: ReadOnlyEqualSelectorFamilyOptions<T, P> | ReadWriteEqualSelectorFamilyOptions<T, P>): (param: P) => RecoilValueReadOnly<T> | RecoilState<T> {
  const inner = selectorFamily({
    key: `${options.key}_inner`,
    get: options.get
  });

  const prior = new Map<P, T>();

  return selectorFamily({
    key: options.key,
    get: (param) => ({get}) => {
      const latest = get(inner(param));
      if (prior.has(param) && options.equals(latest, prior.get(param)!)) {
        return prior.get(param)!;
      }
      prior.set(param, latest);
      return latest as T;
    }, ...'set' in options ? {set: options.set} : {}
  });
}
