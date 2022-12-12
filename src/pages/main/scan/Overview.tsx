import {motion} from "framer-motion";
import {Box, Button, Group, Stack, Text, ThemeIcon, useMantineTheme} from "@mantine/core";
import {IconChevronLeft, IconFilter, IconHomeCheck, IconHomeExclamation} from "@tabler/icons";
import React from "react";
import {useRecoilValue, useSetRecoilState} from "recoil";
import {
  actionBatchState,
  applyErrorsState,
  scanPageState,
  selectedAddActionBatchState,
  selectedRemoveActionBatchState
} from "../../../states";
import {ApplyErrors} from "../../../bindings/ApplyErrors";
import {slideFadeAnimation} from "../../../transitions";
import {applyActionBatch, stopFullScan} from "../../../commands";
import {Trans, useTranslation} from "react-i18next";

export const Overview = React.forwardRef(() => {
  const {t} = useTranslation();
  const theme = useMantineTheme();

  const actionBatch = useRecoilValue(actionBatchState);
  const addSelection = useRecoilValue(selectedAddActionBatchState);
  const removeSelection = useRecoilValue(selectedRemoveActionBatchState);
  const setScanPage = useSetRecoilState(scanPageState);
  const setApplyErrors = useSetRecoilState(applyErrorsState);

  const totalItems = (actionBatch?.add.length ?? 0) + (actionBatch?.remove.length ?? 0);
  const selectedItems = addSelection.length + removeSelection.length;

  const apply = async () => {
    setScanPage("applying");
    try {
      await applyActionBatch({
        add: addSelection,
        remove: removeSelection,
      });
      setApplyErrors(null);
    } catch (_e: any) {
      const e = _e as ApplyErrors;
      setApplyErrors(e);
    }
    setScanPage("done");
  }

  return (
    <motion.div key={"overview"} style={{height: "100%"}} {...slideFadeAnimation}>
      <Stack py={"xl"} sx={{height: "100%"}}>
        <Button pos={"absolute"} size={"xs"} sx={{boxShadow: "none"}} variant={"subtle"}
                leftIcon={<IconChevronLeft size={16} strokeWidth={1}/>}
                onClick={stopFullScan}>
          {t('restart')}
        </Button>
        <Box sx={{flexGrow: 1}}/>
        <Group sx={{width: "100%"}} position={"center"}>
          <Group position={"center"} mr={"xl"}>
            <ThemeIcon size={128} radius={64}
                       variant={"gradient"}
                       gradient={(selectedItems > 0) ?
                         {from: "orange", to: "yellow"} :
                         {from: "green", to: "lime"}}>
              {(selectedItems > 0) ?
                <IconHomeExclamation size={72} strokeWidth={1}/> :
                <IconHomeCheck size={72} strokeWidth={1}/>}
            </ThemeIcon>
          </Group>
          <Stack spacing={"xs"}>
            <Text size={20}>{t('scan_complete')}</Text>
            {(totalItems > 0) ? <>
              <Group align={"end"} spacing={"xs"}>
                <Text size={28} color={theme.colorScheme === "dark" ? theme.colors.blue[2] : theme.colors.blue[5]}>
                  {t('items', {count: selectedItems})}
                </Text>
                <Text size={"xs"} color={"dimmed"} pb={4}>{t('selected')}</Text>
              </Group>
              <Group align={"center"}>
                <Button size={"xs"} variant={"light"} sx={{boxShadow: "none"}}
                        onClick={() => setScanPage("detail")}>
                  {t('view_items')}
                </Button>
                <Text size={"xs"} color={"dimmed"}>{t('items_found', {count: totalItems})}</Text>
              </Group>
            </> : <Text size={"sm"}>
              <Trans
                i18nKey="everything_looks_good_no_files_need_to_be_excluded"
                components={{
                  b: <br/>
                }}
              />
            </Text>}
          </Stack>
        </Group>
        {(selectedItems > 0) && <Stack align={"center"} spacing={"xs"} mt={"xl"}>
          <Button variant={"gradient"} leftIcon={<IconFilter/>} onClick={apply}>{t('apply')}</Button>
          <Text size={"xs"} color={"dimmed"}>
            {t("exclude_include_selected_files_from_timemachine_backups")}
          </Text>
        </Stack>}
        <Box sx={{flexGrow: 1}}/>
      </Stack>
    </motion.div>)
});
