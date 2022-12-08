import {createBrowserRouter, RouterProvider} from "react-router-dom";
import {SWRConfig} from "swr";
import {MantineProvider} from "@mantine/core";
import {RecoilRoot} from "recoil";
import {useColorScheme} from "@mantine/hooks";
import {disableMenu} from "./utils";
import {MainLayout} from "./pages/main/MainLayout";
import {Stats} from "./pages/main/Stats";
import {SyncActionBatch} from "./states";
import {Directories} from "./pages/main/Directories";
import {General} from "./pages/main/General";
import {Scan} from "./pages/main/Scan";
import {Rules} from "./pages/main/Rules";
import {About} from "./pages/About";
import {Ack} from "./pages/Ack";
import {License} from "./pages/License";

const router = createBrowserRouter([
  {
    path: "main",
    element: <MainLayout/>,
    children: [
      {
        path: "stats",
        element: <Stats/>
      },
      {
        path: "directories",
        element: <Directories/>
      },
      {
        path: "general",
        element: <General/>
      },
      {
        path: "rules",
        element: <Rules/>
      },
      {
        path: "scan",
        element: <Scan/>,
      },
    ]
  },
  {
    path: "about",
    element: <About/>
  },
  {
    path: "ack",
    element: <Ack/>
  },
  {
    path: "license",
    element: <License/>
  }
]);

export const App = () => {
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
            },
            Navbar: {
              styles: {
                root: {
                  zIndex: 250
                }
              }
            },
            Header: {
              styles: {
                root: {
                  zIndex: 251
                }
              }
            }
          }
        }}
      >
        <RecoilRoot>
          <SyncActionBatch/>
          <RouterProvider router={router}/>
        </RecoilRoot>
      </MantineProvider>
    </SWRConfig>
  )
}
