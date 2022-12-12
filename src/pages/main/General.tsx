import {Box, Checkbox, Container, Group, Select, Stack, Text} from "@mantine/core";
import {useRecoilState} from "recoil";
import {autoStartState, languageState, noIncludeState} from "../../states";
import {useTranslation} from "react-i18next";
import {availableLanguages} from "../../i18n";

export const General = () => {
  const {t} = useTranslation();

  const [noInclude, setNoInclude] = useRecoilState(noIncludeState);
  const [autoStart, setAutoStart] = useRecoilState(autoStartState);
  const [language, setLanguage] = useRecoilState(languageState);

  return (<Container>
    <Stack py={"xl"}>
      <Checkbox size={"sm"} label={<Text size={"md"}>{t('start_at_login')}</Text>}
                checked={autoStart} onChange={(ev) => {
        setAutoStart(ev.currentTarget.checked);
      }}/>
      <Group>
        <Text>{t('language')}</Text>
        <Select
          data={availableLanguages(t)}
          value={language}
          size={"sm"}
          onChange={(lang) => setLanguage(lang ?? "auto")}
        />
      </Group>
      <Box/>
      <Checkbox
        checked={noInclude}
        size={"sm"}
        onChange={() => {
          setNoInclude(!noInclude);
        }}
        label={<>
          <Text size={"md"}>{t('ignore_included_files')}</Text>
          <Text size={"sm"} color={"dimmed"}>
            {t('dont_include_files_into_backups_even_if_they_dont')}
          </Text>
        </>}/>
    </Stack>
  </Container>)
};