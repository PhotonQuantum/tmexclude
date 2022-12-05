import {Box, Container, Group, ScrollArea, Table, Tabs, Text} from "@mantine/core";
import {cargoLicenses, License, npmLicenses} from "../licenses";
import {useViewportSize} from "@mantine/hooks";

const A = (props: { href: string, children: React.ReactNode }) => (
  <Text inline sx={{cursor: "pointer"}} c={"blue"} component={"a"} target={"_blank"} {...props}/>
)

const DepTab = ({deps}: { deps: Array<License> }) => (
  <ScrollArea sx={{height: "100%"}}>
    <Table highlightOnHover>
      <thead>
      <tr>
        <th>Package</th>
        <th>License</th>
      </tr>
      </thead>
      <tbody>
      {
        deps.map(({name, license, repository, version}) => (
          <tr key={name}>
            <td>
              <Group spacing={"xs"}>
                {
                  repository !== null ?
                    <A href={repository}>{name}</A>
                    :
                    <Text>{name}</Text>
                }
                {version && <Text span inline inherit color={"dimmed"}>{version}</Text>}
              </Group>
            </td>
            <td><Text>{license}</Text></td>
          </tr>
        ))
      }
      </tbody>
    </Table>
  </ScrollArea>
)

export const Ack = () => {
  const {height: vh} = useViewportSize();
  return (
    <Box sx={{height: vh}}>
      <Container sx={{height: "100%"}} py={"sm"}>
        <Tabs defaultValue={"cargo"} sx={{display: "flex", flexDirection: "column", height: "100%"}} styles={{
          panel: {
            overflowY: "hidden"
          }
        }}>
          <Tabs.List>
            <Tabs.Tab value={"cargo"}>Cargo</Tabs.Tab>
            <Tabs.Tab value={"npm"}>NPM</Tabs.Tab>
          </Tabs.List>
          <Tabs.Panel value={"cargo"} pt={"xs"}>
            <DepTab deps={cargoLicenses}/>
          </Tabs.Panel>
          <Tabs.Panel value={"npm"} pt={"xs"}>
            <DepTab deps={npmLicenses.map(({name, ...fields}) => {
                return {
                  name: name.split("@").slice(0, -1).join("@"),
                  version: name.split("@").slice(-1),
                  ...fields
                } as License
              }
            )}/>
          </Tabs.Panel>
        </Tabs>
      </Container>
    </Box>
  )
}