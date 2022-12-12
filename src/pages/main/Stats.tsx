import {Box, Card, Container, Group, Stack, Text, ThemeIcon, Title} from "@mantine/core";
import {IconCheck, IconDots, IconFileMinus, IconFilePlus, IconRadar} from "@tabler/icons";
import useSWR from 'swr'
import {Metrics} from "../../bindings/Metrics";
import ReactTimeago from "react-timeago";
import {PathText} from "../../components/PathText";
import {swrFetcher} from "../../utils";
import {Trans, useTranslation} from "react-i18next";
import {zh_CN_formatter} from "../../i18n";

export const Stats = () => {
  const {t, i18n} = useTranslation();

  const {data} = useSWR<Metrics>("metrics", swrFetcher);

  const formatter = i18n.language === "zh-CN" ? zh_CN_formatter : undefined;

  return (<Container>
    <Stack py={"xl"}>
      <Box pl={"xl"} pb={"xl"}>
        <Title order={2}>{t('looks_good')}</Title>
        <Text size={"sm"}>{t('timemachine_exclude_is_running')}</Text>
      </Box>
      <Card radius={"lg"} withBorder>
        <Group>
          <ThemeIcon radius={"md"} size={50} variant={"gradient"}
                     gradient={{from: "cyan", to: "indigo", deg: 105}}>
            <IconFilePlus size={30} strokeWidth={1.5}/>
          </ThemeIcon>
          <Box>
            <Group spacing={"xs"}>
              <ThemeIcon size={16} variant={"outline"} radius={"xl"} color={"green"}>
                <IconCheck size={12} strokeWidth={3}/>
              </ThemeIcon>
              <Text size={"xl"}>{t('files', {'count': data?.["files-excluded"]})}</Text>
            </Group>
            <Text size={"sm"} color={"dimmed"}>{t('have_been_excluded_from_timemachine_backups')}</Text>
          </Box>
        </Group>
      </Card>
      <Card radius={"lg"} withBorder>
        <Group>
          <ThemeIcon radius={"md"} size={50} variant={"gradient"}
                     gradient={{from: "lime", to: "teal", deg: 105}}>
            <IconFileMinus size={30} strokeWidth={1.5}/>
          </ThemeIcon>
          <Box>
            <Group spacing={"xs"}>
              <ThemeIcon size={16} variant={"outline"} radius={"xl"} color={"green"}>
                <IconCheck size={12} strokeWidth={3}/>
              </ThemeIcon>
              <Text size={"xl"}>{t('files', {'count': data?.["files-included"]})}</Text>
            </Group>
            <Text size={"sm"} color={"dimmed"}>{t('have_been_reincluded_into_timemachine_backups')}</Text>
          </Box>
        </Group>
      </Card>
      <Card radius={"lg"} withBorder>
        <Group>
          <ThemeIcon radius={"md"} size={50} variant={"gradient"}
                     gradient={{from: "yellow", to: "pink", deg: 105}}>
            <IconRadar size={30} strokeWidth={1.5}/>
          </ThemeIcon>
          <Box>
            <Group spacing={"xs"}>
              <ThemeIcon size={16} variant={"outline"} radius={"xl"} color={"orange"}>
                <IconDots size={12} strokeWidth={3}/>
              </ThemeIcon>
              <PathText path={data?.["last-excluded"] ? data?.["last-excluded"] : "N/A"} size={"xl"} lineClamp={1}
                        keepFirst={4} keepLast={2}/>
            </Group>
            <Text size={"sm"} color={"dimmed"}>{(data && data["last-excluded-time"] !== 0) ? <span>
              <Trans i18nKey={"was_excluded"}>
                was excluded <ReactTimeago formatter={formatter} date={data["last-excluded-time"] * 1000}/>
              </Trans>
              </span> : t("no_files_have_been_excluded_yet")}</Text>
          </Box>
        </Group>
      </Card>
    </Stack>
  </Container>)
};

