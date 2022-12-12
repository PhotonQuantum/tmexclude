import {
  Badge,
  Box,
  Button,
  Container,
  createStyles,
  MultiSelect,
  Popover,
  ScrollArea,
  Stack,
  Table,
  Text
} from "@mantine/core";
import {IconMinus, IconPlus} from "@tabler/icons";
import React, {useState} from "react";
import {useRecoilState, useRecoilValue, useSetRecoilState} from "recoil";
import {dirsState, perDirState, ruleNamesState, skipsState} from "../../states";
import {open} from "@tauri-apps/api/dialog";
import _ from "lodash";
import {createScopedKeydownHandler} from "@mantine/utils";
import {PathText} from "../../components/PathText";
import {useTranslation} from "react-i18next";

const buttonStyles = {
  root: {paddingRight: 7},
  leftIcon: {marginRight: 0}
};

const useStyles = createStyles((theme) => ({
  rowSelected: {
    backgroundColor: theme.colorScheme === 'dark' ?
      theme.fn.rgba(theme.colors[theme.primaryColor][7], 0.2) :
      theme.colors[theme.primaryColor][0],
  },
}));

type WatchedDirItemProps = {
  path: string, rules: string[], ruleNames: string[]
}

const WatchedDirItem = React.memo(({
                                     path,
                                     rules,
                                     ruleNames
                                   }: WatchedDirItemProps) => {
  const {t} = useTranslation();

  const setValue = useSetRecoilState(perDirState(path));
  const setDirs = useSetRecoilState(dirsState);
  const [popOverOpened, setPopOverOpened] = useState(false);

  const removeDir = () => {
    setDirs((dirs) => {
      return dirs.filter((dir) => dir.path !== path);
    });
  }
  return (<Popover withArrow trapFocus withinPortal shadow={"sm"} width={500}
                   opened={popOverOpened} onChange={setPopOverOpened}>
    <Popover.Target>
      <Box component="tr" key={path} sx={{cursor: "pointer"}} tabIndex={0}
           onClick={() => setPopOverOpened(true)}
        //@ts-ignore
           onKeyDown={createScopedKeydownHandler({
             parentSelector: "tbody",
             siblingSelector: "tr",
             orientation: "vertical",
             onKeyDown: (e) => {
               if (e.key === "Enter" || e.key == " ") {
                 setPopOverOpened(true);
               }
             }
           })}>
        <td>
          <PathText keepFirst={3} keepLast={1} path={path} lineClamp={1} sx={{
            cursor: "pointer",
            minWidth: 70
          }}/>
        </td>
        <td>{rules.map((rule) => (<Badge key={rule} variant={"light"}>
          <Text size={9} sx={{cursor: "pointer"}}>{rule}</Text>
        </Badge>))}</td>
      </Box>
    </Popover.Target>
    <Popover.Dropdown>
      <>
        <Stack spacing={"xs"}>
          <MultiSelect
            searchable
            data={ruleNames}
            value={rules}
            maxDropdownHeight={150}
            placeholder={"Pick rules to apply"}
            onChange={(rules) => {
              setValue((value) => {
                return {
                  ...value,
                  rules: rules
                };
              })
            }}
          />
          <Button color={"red"} onClick={removeDir}>{t('remove_directory')}</Button>
        </Stack>
      </>
    </Popover.Dropdown>
  </Popover>)
}, _.isEqual);

const WatchedDir = () => {
  const {t} = useTranslation();

  const ruleNames = useRecoilValue(ruleNamesState);
  const [dirs, setDirs] = useRecoilState(dirsState);

  const addDir = async () => {
    const selected = await open({
      directory: true
    });
    if (typeof selected === "string") {
      setDirs((dirPaths) => {
        if (dirPaths.find((dir) => dir.path === selected)) {
          return dirPaths;
        }
        return [...dirPaths, {
          path: selected,
          rules: []
        }];
      });
    }
  }
  return (<>
    <Text>{t('directories_to_watch_and_scan')}</Text>
    <ScrollArea sx={{flexGrow: 1, flexBasis: 0}}>
      <Table highlightOnHover>
        <tbody>
        {dirs.map(({
                     path,
                     rules
                   }) => (<WatchedDirItem key={path} path={path} rules={rules} ruleNames={ruleNames}/>))}
        </tbody>
      </Table>
    </ScrollArea>
    <Button ml={"auto"} size={"xs"} compact leftIcon={<IconPlus size={12}/>} styles={buttonStyles}
            variant={"default"}
            onClick={addDir}
    />
  </>)
}

const SkippedDirs = () => {
  const {t} = useTranslation();

  const [skipped, setSkipped] = useRecoilState(skipsState);
  const [selected, setSelected] = useState(-1);
  const {
    classes,
    cx
  } = useStyles();

  const addDir = async () => {
    const selected = await open({
      directory: true
    });
    if (typeof selected === "string") {
      setSkipped((skipped) => {
        if (skipped.includes(selected)) {
          return skipped;
        }
        return [...skipped, selected];
      });
    }
  }
  const mayRemove = selected >= 0 && selected < skipped.length;
  const removeDir = (index: number) => {
    setSkipped((skipped) => {
      return skipped.filter((_, i) => i !== index);
    });
  };
  return (<>
    <Text>{t('skip_the_following_paths')}</Text>
    <ScrollArea sx={{flexGrow: 1, flexBasis: 0}} onKeyDown={(e) => {
      if (e.key in ["ArrowUp", "ArrowDown", " "]) {
        e.preventDefault();
      }
    }}>
      <Table highlightOnHover>
        <tbody>
        {skipped.map((path, idx) => (<tr key={path}
                                         tabIndex={0}
                                         className={cx({[classes.rowSelected]: selected === idx})}
          // @ts-ignore
                                         onKeyDown={createScopedKeydownHandler({
                                           parentSelector: "tbody",
                                           siblingSelector: "tr",
                                           orientation: "vertical",
                                           onKeyDown: (e) => {
                                             if (e.key === "Enter" || e.key == " ") {
                                               setSelected(idx);
                                             }
                                           }
                                         })}
                                         onClick={() => setSelected(idx)}>
          <td>
            <PathText keepFirst={3} keepLast={1} path={path} lineClamp={1}/>
          </td>
        </tr>))}
        </tbody>
      </Table>
    </ScrollArea>
    <Button.Group ml={"auto"}>
      <Button size={"xs"} compact leftIcon={<IconPlus size={12}/>} styles={buttonStyles} variant={"default"}
              onClick={addDir}/>
      <Button size={"xs"} compact leftIcon={<IconMinus size={12}/>} styles={buttonStyles} variant={"default"}
              disabled={!mayRemove}
              onClick={() => {
                removeDir(selected);
                setSelected(-1);
              }}/>
    </Button.Group>
  </>)
}

export const Directories = () => {
  return (<Container sx={{height: "100%"}}>
    <Stack py={"xl"} spacing={"xs"} sx={{height: "100%"}}>
      <WatchedDir/>
      <SkippedDirs/>
    </Stack>
  </Container>)
}