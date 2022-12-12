import {motion} from "framer-motion";
import {ActionIcon, Card, Group, Stack, Text, useMantineTheme} from "@mantine/core";
import {IconAnalyze, IconHomeSearch, IconRefreshAlert} from "@tabler/icons";
import React from "react";
import {fadeAnimation} from "../../../transitions";
import {startFullScan} from "../../../commands";
import {useTranslation} from "react-i18next";

export const Welcome = React.forwardRef(() => {
  const {t} = useTranslation();
  const theme = useMantineTheme();

  return (
    <motion.div key={"welcome"} style={{height: "100%"}} {...fadeAnimation}>
      <Stack py={"xl"} align={"center"} justify={"center"} sx={{height: "100%"}}>
        <ActionIcon size={128} radius={64} variant={"gradient"} onClick={startFullScan}
                    sx={{boxShadow: theme.shadows.xl}}>
          <IconHomeSearch size={72} strokeWidth={1}/>
        </ActionIcon>
        <Text size={"xl"}>{t('run_a_manual_scan')}</Text>
        <Card>
          <Stack>
            <Group>
              <IconAnalyze size={24} strokeWidth={1.5}/>
              <Text size={"sm"}>{t('run_an_initial_full_scan_after_setup')}</Text>
            </Group>
            <Group>
              <IconRefreshAlert size={24} strokeWidth={1.5}/>
              <Text size={"sm"}>{t('resync_file_changes_if_incremental_scans_fail')}</Text>
            </Group>
          </Stack>
        </Card>
      </Stack>
    </motion.div>)
});
