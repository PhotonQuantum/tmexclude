'use client';
import {motion} from "framer-motion";
import {useRecoilValue} from "recoil";
import {scanCurrentState} from "../../../states";
import {ActionIcon, Stack, Text, ThemeIcon, useMantineTheme} from "@mantine/core";
import {IconSearch, IconSquare} from "@tabler/icons";
import React from "react";
import {useAnimateStyles} from "../../../utils";
import {fadeAnimation} from "../../../transitions";
import {PathText} from "../../../components/PathText";
import {stopFullScan} from "../../../commands";

export const InProgress = React.forwardRef(() => {
  const {
    found,
    path
  } = useRecoilValue(scanCurrentState);
  const theme = useMantineTheme();
  const {classes} = useAnimateStyles();
  const moreDimmed = theme.colorScheme === 'dark' ? theme.colors.dark[3] : theme.colors.gray[5];

  return (
    <motion.div key={"inProgress"} style={{height: "100%"}} {...fadeAnimation}>
      <Stack py={"xl"} align={"center"} justify={"center"} sx={{height: "100%"}}>
        <ThemeIcon size={128} radius={64} variant={"gradient"}>
          <IconSearch size={72} strokeWidth={1} className={classes.circle}/>
        </ThemeIcon>
        <Stack spacing={"xs"} align={"center"}>
          <Text size={"xl"}>Scanning system...</Text>
          <PathText size={"sm"} color={moreDimmed} align={"center"} lineClamp={1} keepFirst={4} keepLast={2}
                    path={path}/>
          <Text size={"sm"} color={"dimmed"}>Found {found} file(s)</Text>
          <ActionIcon variant={"default"} radius={16} size={32} onClick={stopFullScan}>
            <IconSquare size={16} strokeWidth={1.5}/>
          </ActionIcon>
        </Stack>
      </Stack>
    </motion.div>)
});
