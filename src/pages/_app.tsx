import type {AppProps} from "next/app";

import {MantineProvider} from "@mantine/core";
import Head from "next/head";
import {useColorScheme} from "@mantine/hooks";
import {ReactElement, ReactNode} from "react";
import {NextPage} from "next";
import {AnimatePresence} from "framer-motion";
import {SWRConfig} from "swr";
import {disableMenu} from "../utils";
import {RecoilRoot} from "recoil";

export type NextPageWithLayout<P = {}, IP = P> = NextPage<P, IP> & {
    getLayout?: (page: ReactElement) => ReactNode
}

type AppPropsWithLayout = AppProps & {
    Component: NextPageWithLayout
}

// This default export is required in a new `pages/_app.js` file.
export default function MyApp({Component, pageProps}: AppPropsWithLayout) {
    const preferredColorScheme = useColorScheme();
    const getLayout = Component.getLayout ?? ((page) => page);

    disableMenu();

    return (
        <>
            <Head>
                <title>tmexclude</title>
                <meta name="viewport" content="minimum-scale=1, initial-scale=1, width=device-width"/>
            </Head>

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
                                    root:
                                        {
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
                        <AnimatePresence mode={"wait"} initial={false}>
                            {getLayout(<Component {...pageProps} />)}
                        </AnimatePresence>
                    </RecoilRoot>
                </MantineProvider>
            </SWRConfig>
        </>
    )
}
