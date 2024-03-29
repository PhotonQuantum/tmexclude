import {useEffect, useLayoutEffect, useRef, useState} from "react";
import {createStyles, keyframes} from "@mantine/core";

export const disableMenu = () => {
  if (typeof window === "undefined") {
    return;
  }

  // @ts-ignore
  if (window.__TAURI__.environment !== 'production') {
    return
  }

  document.addEventListener('contextmenu', e => {
    e.preventDefault();
    return false;
  }, {capture: true})

  document.addEventListener('selectstart', e => {
    e.preventDefault();
    return false;
  }, {capture: true})
}

function _getHomeDirPath() {
  let result: string | null = null;
  return async () => {
    if (result !== null) {
      return result;
    }
    if (typeof window === "undefined") {
      return "!!!!!!!!!"
    }
    const homeDir = await import("@tauri-apps/api/path").then(path => path.homeDir);
    let dir = await homeDir();
    // @ts-ignore
    result = dir;
    return dir;
  }
}

const getHomeDirPath = _getHomeDirPath();

export const truncatePath: (path: string, keepFirst: number,
                            keepLast: number) => Promise<[boolean, string]> = async (path, keepFirst, keepLast) => {
  const homeReplaced = path.replace(await getHomeDirPath(), "~/");
  let changed = homeReplaced !== path;
  path = homeReplaced;

  const parts = path.split('/');
  if (parts.length <= keepFirst + keepLast) {
    return [changed, path];
  }
  return [true, parts.slice(0, keepFirst).join('/') + '/.../' + parts.slice(-keepLast).join('/')];
}

export const useTruncatedPath = (path: string, keepFirst: number, keepLast: number) => {
  const [truncated, setTruncated] = useState<[boolean, string]>([false, path]);
  useEffect(() => {
    truncatePath(path, keepFirst, keepLast).then(setTruncated);
  }, [path, keepFirst, keepLast]);
  return truncated;
}

export const useIsOverflow = <T extends HTMLElement = any>() => {
  const ref = useRef<T>(null);
  const [isOverflow, setIsOverflow] = useState(false);

  useLayoutEffect(() => {
    const {current} = ref;

    if (current) {
      const hasOverflow = current.scrollHeight > current.clientHeight;

      setIsOverflow(hasOverflow);
    }
  }, [ref.current]);

  return {ref, isOverflow};
};

export const useTableStyles = createStyles((theme) => ({
  rowSelected: {
    backgroundColor:
      theme.colorScheme === 'dark'
        ? theme.fn.rgba(theme.colors[theme.primaryColor][7], 0.2)
        : theme.colors[theme.primaryColor][0],
  },
  stickyHeader: {
    position: 'sticky',
    top: 0,
    backgroundColor: theme.colorScheme === 'dark' ? theme.colors.dark[7] : theme.white,
  },
}))

const circling = (radius: number) => keyframes({
  "from": {transform: `rotate(0deg) translateX(${radius}px) rotate(0deg)`},
  "to": {transform: `rotate(360deg) translateX(${radius}px) rotate(-360deg)`}
})

export const useAnimateStyles = createStyles({
  circle: {
    animation: `${circling(10)} 2s linear infinite`,
  }
})

export const swrFetcher = async (key: string) => {
  const invoke = await import("@tauri-apps/api").then(tauri => tauri.invoke);
  return await invoke<any>(key);
}

export const evDrag = async (ev: { preventDefault: () => void; }) => {
  const {appWindow} = await import("@tauri-apps/api/window");
  ev.preventDefault();
  await appWindow.startDragging();
};
