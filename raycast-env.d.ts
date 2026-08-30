/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {
  /** Search results limit - Maximum number of items shown in the list. The lower the number of shown items is, the faster they'll be retrieved. Defaults to 100. Supports up to 1000 items. */
  "searchResultLimitStr": string,
  /** View Type - Type of view to use to display the results. */
  "viewType": "list" | "grid",
  /** Grid Item Size - Control the size of individual grid cells in the grid view. */
  "gridItemSize": "small" | "medium" | "large",
  /** Data Set - The data set to use for the search. By default it uses the minimal data set, which contains about 5400 characters. The full set contains +40000 characters. Using this big of a dataset might impact performance. Note: In 'List View' only the minimal dataset is supported. */
  "dataSet": "minimal" | "full",
  /** Primary Action - The action to perform when selecting an item. */
  "primaryAction": "paste" | "copy"
}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `index` command */
  export type Index = ExtensionPreferences & {}
}

declare namespace Arguments {
  /** Arguments passed to the `index` command */
  export type Index = {}
}

