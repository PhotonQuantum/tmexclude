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

export const truncatePath: (path: string) => Promise<[boolean, string]> = async (path: string) => {
  const homeReplaced = path.replace(await getHomeDirPath(), "~/");
  let changed = homeReplaced !== path;
  path = homeReplaced;

  const parts = path.split('/');
  if (parts.length <= 4) {
    return [changed, path];
  }
  return [true, parts.slice(0, 3).join('/') + '/.../' + parts.slice(-1)];
}