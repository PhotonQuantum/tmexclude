'use client';
import {motion} from "framer-motion";
import {ActionIcon, Card, Group, Stack, Text, useMantineTheme} from "@mantine/core";
import {IconAnalyze, IconHomeSearch, IconRefreshAlert} from "@tabler/icons";
import React from "react";
import {fadeAnimation} from "../../../transitions";
import {startFullScan} from "../../../commands";

export const Welcome = React.forwardRef(() => {
  const theme = useMantineTheme();
  return (
    <motion.div key={"welcome"} style={{height: "100%"}} {...fadeAnimation}>
      <Stack py={"xl"} align={"center"} justify={"center"} sx={{height: "100%"}}>
        <ActionIcon size={128} radius={64} variant={"gradient"} onClick={startFullScan}
                    sx={{boxShadow: theme.shadows.xl}}>
          <IconHomeSearch size={72} strokeWidth={1}/>
        </ActionIcon>
        <Text size={"xl"}>Run a manual scan</Text>
        <Card>
          <Stack>
            <Group>
              <IconAnalyze size={24} strokeWidth={1.5}/>
              <Text size={"sm"}>Run an initial full scan after setup.</Text>
            </Group>
            <Group>
              <IconRefreshAlert size={24} strokeWidth={1.5}/>
              <Text size={"sm"}>Re-sync file changes if incremental scans fail.</Text>
            </Group>
          </Stack>
        </Card>
      </Stack>
    </motion.div>)
});
