'use client';
import {MantineProvider} from "@mantine/core";
import {RecoilRoot} from "recoil";
import {SyncActionBatch} from "./states";
import {SWRConfig} from "swr";
import {useColorScheme} from "@mantine/hooks";
import {disableMenu} from "../utils";
import {ReactNode} from "react";

export const Providers = ({children}: { children: ReactNode }) => {
  const preferredColorScheme = useColorScheme();
  disableMenu();
  return (
    <SWRConfig
      value={{
        refreshInterval: 5000,
        refreshWhenOffline: true,
        revalidateOnReconnect: false
      }}
    >
      <MantineProvider
        withGlobalStyles
        withNormalizeCSS
        theme={{
          colorScheme: preferredColorScheme,
          components: {
            Text: {
              styles: {
                root: {
                  userSelect: "none",
                  cursor: "default",
                }
              }
            },
            Input: {
              defaultProps: {
                spellCheck: false
              }
            },
            Title: {
              styles: {
                root: {
                  userSelect: "none",
                  cursor: "default",
                }
              }
            },
            ScrollArea: {
              styles: (theme) => ({
                root: {
                  maxHeight: "100%",
                  borderStyle: "solid",
                  borderWidth: "1px",
                  borderRadius: theme.radius.xs,
                  borderColor: theme.colorScheme === 'dark' ? theme.colors.dark[4] : theme.colors.gray[2]
                },
              })
            },
            Button: {
              styles: (theme) => ({
                root: {
                  boxShadow: theme.shadows.xs,
                }
              })
            }
          }
        }}
      >
        <RecoilRoot>
          <SyncActionBatch/>
          {children}
        </RecoilRoot>
      </MantineProvider>
    </SWRConfig>
  )
}