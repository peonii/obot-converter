// place files you want to import through the `$lib` alias in this folder.

import { Format } from './pkg/obot_converter'
import { GameVersion } from './pkg/obot_converter_bg'

export * from './pkg/obot_converter'

export const formats: {
    [key: number]: [string, string, number]
} = {
    [Format.OmegaBot]: ['OmegaBot 3', 'replay', GameVersion.Version2113],
    [Format.OmegaBot2]: ['OmegaBot 2', 'replay', GameVersion.Version2113],
    [Format.URL]: ['OmegaBot 1', 'replay', GameVersion.Version2113],
    [Format.Tasbot]: ['TASBot', 'json', GameVersion.Version2113],
    [Format.MHRBinary]: ['MHR (Binary)', 'mhr', GameVersion.Version2113],
    [Format.MHRJson]: ['MHR (Json)', 'mhr.json', GameVersion.Version2113],
    [Format.EchoOld]: ['Echo (Old, Json)', 'echo', GameVersion.Version2113],
    [Format.EchoNewJson]: ['Echo (New, Json)', 'echo', GameVersion.Version2113],
    [Format.EchoNewBinary]: ['Echo (New, Binary)', 'echo', GameVersion.Version2113],
    [Format.ZBot]: ['zBot', 'zbf', GameVersion.Version2113],
    [Format.KDBot]: ['KD-Bot', 'kd', GameVersion.Version2113],
    [Format.Rush]: ['Rush', 'rsh', GameVersion.Version2113],
    [Format.ReplayBot]: ['ReplayBot', 'replay', GameVersion.Version2113],
    [Format.Fembot]: ['Fembot', 'freplay', GameVersion.Version2113],
    [Format.XBot]: ['xBot', 'xbot', GameVersion.Version2113],
    [Format.YBot1]: ['yBot 1', '', GameVersion.Version2113],
    [Format.YBot]: ['yBot 2', 'ybot', GameVersion.Version2206],
    [Format.XDBot]:['XDBot', 'xd', GameVersion.Version2206],
    [Format.GDR]: ['GDR', 'gdr', GameVersion.Version2206],
    [Format.GDRJson]: ['GDR (Json)', 'gdr.json', GameVersion.Version2206],
    [Format.PlainText]: ['Plain Text', 'txt', GameVersion.Any],
}
