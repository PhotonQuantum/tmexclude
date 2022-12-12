import i18n, {LanguageDetectorAsyncModule, TFunction} from "i18next";
import {initReactI18next} from "react-i18next";
import en from "./i18n/en.json";
import zh_hans from "./i18n/zh_hans.json";
// @ts-ignore
import zh_CN_strings from "react-timeago/lib/language-strings/zh-CN";
// @ts-ignore
import buildFormatter from "react-timeago/lib/formatters/buildFormatter";
import {getStore} from "./commands";

export const codeToDisplayName = (languageCode: string, t: TFunction) => {
  if (languageCode === "auto") {
    return t("lang_auto")
  }
  const intl_display = new Intl.DisplayNames([languageCode], {type: "language"});
  return intl_display.of(languageCode);
}

const supported = ["en", "zh-Hans"];

export const availableLanguages = (t: TFunction) => (["auto", ...supported].map(code => ({
  value: code, label: codeToDisplayName(code, t)
})));

export const zh_CN_formatter = buildFormatter(zh_CN_strings);

const storeDetector: LanguageDetectorAsyncModule = {
  type: "languageDetector",
  async: true,
  detect: async () => {
    const language = await getStore("language") ?? "auto";
    if (language === "auto") {
      return navigator.languages
    }
    return language;
  }
}

i18n
  .use(storeDetector)
  .use(initReactI18next)
  .init({
    fallbackLng: {
      "zh-CN": ["zh-Hans", "en"],
      default: ["en"]
    },
    interpolation: {
      escapeValue: false
    },
    resources: {
      en: {
        translation: en
      },
      "zh-Hans": {
        translation: zh_hans
      }
    }
  });

export default i18n;