import { readdir } from "node:fs/promises";

const schemaify = async (path: string) => {
  console.log("Schemaifying " + path);
  for (const entry of await readdir(path, { withFileTypes: true })) {
    if (entry.isDirectory() && entry.name !== "node_modules") {
      await schemaify(path + "/" + entry.name);
    } else if (entry.name === "keyboard.json") {
      const filepath = path + "/" + entry.name;
      console.log(filepath);
      let file = Bun.file(filepath);
      let object = JSON.parse(await file.text());
      object["$schema"] =
        "https://raw.githubusercontent.com/justDeeevin/NuhxBoard/refs/heads/main/schemas/layout.json";
      await Bun.write(file, JSON.stringify(object, null, 2));
    }
  }
};

await schemaify(".");
