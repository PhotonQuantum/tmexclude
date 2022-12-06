import {Container, ScrollArea, Textarea} from "@mantine/core";
// @ts-ignore
import license from "../../LICENSE.txt";
import {useViewportSize} from "@mantine/hooks";

export const License = () => {
  const {height: vh} = useViewportSize();
  return (
    <Container sx={{height: vh}} py={"xs"}>
      <ScrollArea sx={{height: "100%"}}>
        <Textarea size={"xs"} variant={"unstyled"} autosize styles={{input: {textAlign: "center"}}}
                  value={license}
                  onKeyDown={(ev) => ev.preventDefault()} />
      </ScrollArea>
    </Container>)
};