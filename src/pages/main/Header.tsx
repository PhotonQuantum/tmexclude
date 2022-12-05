'use client';
import {useRecoilValue, useResetRecoilState, useSetRecoilState} from "recoil";
import {configChangedState, draftConfigState, finalConfigState} from "../../states";
import {Box, Button, Group, Header, Text} from "@mantine/core";
import {IconAdjustments} from "@tabler/icons";
import React from "react";
import { evDrag } from "../../utils";

export const MainHeader = React.forwardRef<HTMLElement>((props, ref) => {
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
        <Text size={"md"}>Preference</Text>
        {changed && <>
          <Box sx={{flexGrow: 1}}/>
          <Button variant={"subtle"} compact sx={{boxShadow: "none"}} onClick={() => resetDraft()}>Reset</Button>
          <Button compact onClick={() => setFinalConfig(draftConfig)}>Save</Button>
        </>}
      </Group>
    </Header>)
});