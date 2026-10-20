import type { KeyConfig_Serialize, Page_Serialize, Profile_Serialize } from "./bindings";

// The `_Serialize` types are what the backend returns: every field is present.
export type Profile = Profile_Serialize;
export type Page = Page_Serialize;
export type KeyConfig = KeyConfig_Serialize;
