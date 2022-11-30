'use client';
import {Group, Loader, Stack, Text, ThemeIcon} from "@mantine/core";
import {IconTool} from "@tabler/icons";
import React from "react";
import {useAnimateStyles} from "../../../utils";
import {fadeAnimation} from "../../../transitions";
import {motion} from "framer-motion";

export const Applying = React.forwardRef(() => {
  const {classes} = useAnimateStyles();
  return (
    <motion.div key={"applying"} style={{height: "100%"}} {...fadeAnimation}>
      <Stack py={"xl"} align={"center"} justify={"center"} sx={{height: "100%"}}>
        <ThemeIcon size={128} radius={64} variant={"gradient"}>
          <IconTool size={72} strokeWidth={1} className={classes.circle}/>
        </ThemeIcon>
        <Stack align={"center"} spacing={"lg"}>
          <Text size={"xl"}>Applying changes...</Text>
          <Group spacing={"xs"} align={"center"}>
            <Loader size={"xs"}/>
            <Text size={"xs"}>Setting file attributes</Text>
          </Group>
        </Stack>
      </Stack>
    </motion.div>)
});