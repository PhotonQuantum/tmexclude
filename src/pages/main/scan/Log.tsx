import {useRecoilValue, useSetRecoilState} from "recoil";
import {applyErrorsState, scanPageState} from "../../../states";
import React from "react";
import {Box, Button, Group, ScrollArea, Stack, Text, ThemeIcon, Tooltip, useMantineTheme} from "@mantine/core";
import {IconAlertTriangle, IconChevronLeft, IconHomeExclamation} from "@tabler/icons";
import {motion} from "framer-motion";
import {fadeAnimation} from "../../../transitions";
import {PathText} from "../../../components/PathText";
import {stopFullScan} from "../../../commands";
import {useTranslation} from "react-i18next";

export const Log = React.forwardRef(() => {
  const {t} = useTranslation();

  const theme = useMantineTheme();

  const applyErrors = useRecoilValue(applyErrorsState);
  const setScanPage = useSetRecoilState(scanPageState);

  const onBack = async () => {
    await stopFullScan();
    setScanPage("scan");
  };

  return (
    <motion.div key={"done"} style={{height: "100%"}} {...fadeAnimation}>
      <Stack py={"xl"} sx={{height: "100%"}}>
        <Button pos={"absolute"} size={"xs"} sx={{boxShadow: "none"}} variant={"subtle"}
                leftIcon={<IconChevronLeft size={16} strokeWidth={1}/>}
                onClick={onBack}>
          {t('restart')}
        </Button>
        <Box sx={{flexGrow: 1}}/>
        <Group sx={{width: "100%"}} position={"center"}>
          <Group position={"center"} mr={"xl"}>
            <ThemeIcon size={128} radius={64}
                       variant={"gradient"}
                       gradient={{from: "orange", to: "yellow"}}>
              <IconHomeExclamation size={72} strokeWidth={1}/>
            </ThemeIcon>
          </Group>
          <Stack sx={{width: 320}}>
            <Text size={24}>{t('apply_log')}</Text>
            <ScrollArea.Autosize maxHeight={200} styles={{root: {borderStyle: "hidden"}}} offsetScrollbars>
              {
                Object.entries(applyErrors?.errors ?? {}).map(([path, reason]) => (
                  <Group key={path} align={"center"} spacing={"xs"} position={"apart"} sx={{minHeight: 50}}>
                    <PathText withinPortal keepFirst={3} keepLast={1} path={path} lineClamp={1}/>
                    <Tooltip multiline width={300} label={reason} withinPortal withArrow arrowSize={12}>
                      <Group spacing={"xs"} ml={"auto"}>
                        <Text color={"dimmed"}>{t('failed')}</Text>
                        <IconAlertTriangle size={16} strokeWidth={1} color={theme.colors.orange[3]}/>
                      </Group>
                    </Tooltip>
                  </Group>
                ))
              }
            </ScrollArea.Autosize>
            <Button size={"xs"} variant={"light"} color={"orange"} mr={"auto"}
                    onClick={() => setScanPage("done")}>
              {t('hide_log')}
            </Button>
          </Stack>
        </Group>
        <Box sx={{flexGrow: 1}}/>
      </Stack>
    </motion.div>)
});
