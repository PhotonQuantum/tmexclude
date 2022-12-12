import {useRecoilValue, useResetRecoilState, useSetRecoilState} from "recoil";
import {configChangedState, draftConfigState, finalConfigState} from "../../states";
import {Box, Button, Group, Header, Text} from "@mantine/core";
import {IconAdjustments} from "@tabler/icons";
import React from "react";
import {evDrag} from "../../utils";
import {useTranslation} from "react-i18next";

export const MainHeader = React.forwardRef<HTMLElement>((props, ref) => {
  const {t} = useTranslation();

  const changed = useRecoilValue(configChangedState);
  const resetDraft = useResetRecoilState(draftConfigState);
  const draftConfig = useRecoilValue(draftConfigState);
  const setFinalConfig = useSetRecoilState(finalConfigState);

  return (
    <Header ref={ref} height={55} p="xs"
            styles={(theme) => ({
              root: {
                background: theme.colorScheme === "dark" ? "#38343C" : "#F6F2F9",
              }
            })}
            onMouseDown={evDrag}>
      <Group spacing={"xs"} p={5} sx={(theme) => ({
        color: theme.colorScheme === "dark" ? "#ffffff" : "inherit",
        alignItems: "flex-start"
      })}>
        <Box sx={{
          height: 20,
          width: 20
        }} pt={2}>
          <IconAdjustments size={20} strokeWidth={1.5}/>
        </Box>
        <Text size={"md"}>{t('preference')}</Text>
        {changed && <>
          <Box sx={{flexGrow: 1}}/>
          <Button variant={"subtle"} compact sx={{boxShadow: "none"}} onClick={() => resetDraft()}>{t('reset')}</Button>
          <Button compact onClick={() => setFinalConfig(draftConfig)}>{t('save')}</Button>
        </>}
      </Group>
    </Header>)
});