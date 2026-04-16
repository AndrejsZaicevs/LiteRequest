/**
 * Postman-compatible dynamic variables ({{$randomX}}, {{$timestamp}}, etc.)
 * Values are generated fresh each time resolveDynamicVars() is called.
 */

// ── Helpers ───────────────────────────────────────────────────

const FIRST_NAMES = ["James","Mary","John","Patricia","Robert","Jennifer","Michael","Linda","William","Barbara","David","Susan","Richard","Jessica","Joseph","Sarah","Thomas","Karen","Charles","Lisa","Christopher","Nancy","Daniel","Betty","Matthew","Margaret","Anthony","Sandra","Mark","Ashley","Donald","Emily","Paul","Dorothy","Steven","Kimberly","Andrew","Carol","Kenneth","Michelle","George","Amanda","Joshua","Melissa","Kevin","Deborah","Brian","Stephanie","Edward","Rebecca"];
const LAST_NAMES = ["Smith","Johnson","Williams","Brown","Jones","Garcia","Miller","Davis","Rodriguez","Martinez","Hernandez","Lopez","Gonzalez","Wilson","Anderson","Thomas","Taylor","Moore","Jackson","Martin","Lee","Perez","Thompson","White","Harris","Sanchez","Clark","Ramirez","Lewis","Robinson","Walker","Young","Allen","King","Wright","Scott","Torres","Nguyen","Hill","Flores","Green","Adams","Nelson","Baker","Hall","Rivera","Campbell","Mitchell","Carter","Roberts"];
const DOMAINS = ["example.com","mail.com","test.org","domain.net","webmail.io","inbox.co","email.dev","sample.app"];
const LOREM_WORDS = ["lorem","ipsum","dolor","sit","amet","consectetur","adipiscing","elit","sed","do","eiusmod","tempor","incididunt","ut","labore","et","dolore","magna","aliqua","enim","ad","minim","veniam","quis","nostrud","exercitation","ullamco","laboris","nisi","aliquip","ex","ea","commodo","consequat"];
const CITIES = ["New York","Los Angeles","Chicago","Houston","Phoenix","Philadelphia","San Antonio","San Diego","Dallas","San Jose","Austin","Jacksonville","Fort Worth","Columbus","Charlotte","Indianapolis","San Francisco","Seattle","Denver","Washington","Nashville","Oklahoma City","El Paso","Boston","Portland","Las Vegas","Memphis","Louisville","Baltimore","Milwaukee"];
const COUNTRIES = ["United States","United Kingdom","Canada","Australia","Germany","France","Japan","India","Brazil","China","Mexico","Italy","Spain","Netherlands","Sweden","Norway","Denmark","Finland","Poland","Argentina","South Korea","Singapore","New Zealand","Switzerland","Austria","Belgium","Portugal","Czech Republic","Hungary","Romania"];
const COUNTRY_CODES = ["US","GB","CA","AU","DE","FR","JP","IN","BR","CN","MX","IT","ES","NL","SE","NO","DK","FI","PL","AR","KR","SG","NZ","CH","AT","BE","PT","CZ","HU","RO"];
const WEEKDAYS = ["Monday","Tuesday","Wednesday","Thursday","Friday","Saturday","Sunday"];
const MONTHS = ["January","February","March","April","May","June","July","August","September","October","November","December"];

const rand = (min: number, max: number) => Math.floor(Math.random() * (max - min + 1)) + min;
const pick = <T>(arr: T[]): T => arr[rand(0, arr.length - 1)];
const hex = () => Math.floor(Math.random() * 16).toString(16);
const uuid = () => "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, c => {
  const r = Math.random() * 16 | 0;
  return (c === "x" ? r : (r & 0x3 | 0x8)).toString(16);
});

// ── Generators ────────────────────────────────────────────────

type Generator = () => string;

export const DYNAMIC_VARS: Record<string, Generator> = {
  // Numbers
  $randomInt:            () => String(rand(0, 1000)),
  $randomFloat:          () => (Math.random() * 1000).toFixed(4),
  $randomBoolean:        () => String(Math.random() < 0.5),
  $randomArrayIndex:     () => String(rand(0, 20)),

  // Identity
  $randomUUID:           uuid,
  $guid:                 uuid,
  $randomAlphaNumeric:   () => Math.random().toString(36).slice(2, 8),

  // Color
  $randomHexColor:       () => `#${hex()}${hex()}${hex()}${hex()}${hex()}${hex()}`,
  $randomRGBColor:       () => `rgb(${rand(0,255)},${rand(0,255)},${rand(0,255)})`,

  // Person
  $randomFirstName:      () => pick(FIRST_NAMES),
  $randomLastName:       () => pick(LAST_NAMES),
  $randomFullName:       () => `${pick(FIRST_NAMES)} ${pick(LAST_NAMES)}`,
  $randomUserName:       () => `${pick(FIRST_NAMES).toLowerCase()}${rand(1, 999)}`,
  $randomNamePrefix:     () => pick(["Mr.","Mrs.","Ms.","Dr.","Prof."]),
  $randomNameSuffix:     () => pick(["Jr.","Sr.","I","II","III","PhD","MD"]),
  $randomJobTitle:       () => pick(["Engineer","Manager","Designer","Developer","Analyst","Director","Consultant","Architect"]),
  $randomJobArea:        () => pick(["Product","Engineering","Marketing","Sales","Finance","Operations","Design","Support"]),

  // Internet
  $randomEmail:          () => `${pick(FIRST_NAMES).toLowerCase()}.${pick(LAST_NAMES).toLowerCase()}${rand(1,99)}@${pick(DOMAINS)}`,
  $randomDomainName:     () => `${Math.random().toString(36).slice(2, 10)}.${pick(["com","net","org","io","dev","app"])}`,
  $randomUrl:            () => `https://${Math.random().toString(36).slice(2, 10)}.${pick(["com","net","org","io"])}/path`,
  $randomIP:             () => `${rand(1,254)}.${rand(0,255)}.${rand(0,255)}.${rand(1,254)}`,
  $randomIPV6:           () => Array.from({length:8}, ()=>rand(0,65535).toString(16).padStart(4,"0")).join(":"),
  $randomMACAddress:     () => Array.from({length:6}, ()=>`${hex()}${hex()}`).join(":"),
  $randomPassword:       () => Math.random().toString(36).slice(2) + Math.random().toString(36).slice(2).toUpperCase(),
  $randomUserAgent:      () => "Mozilla/5.0 (compatible; LiteRequest/1.0)",

  // Location
  $randomCity:           () => pick(CITIES),
  $randomCountry:        () => pick(COUNTRIES),
  $randomCountryCode:    () => pick(COUNTRY_CODES),
  $randomLatitude:       () => (Math.random() * 180 - 90).toFixed(6),
  $randomLongitude:      () => (Math.random() * 360 - 180).toFixed(6),
  $randomStreetAddress:  () => `${rand(1, 9999)} ${pick(LAST_NAMES)} ${pick(["St","Ave","Blvd","Rd","Lane","Drive"])}`,
  $randomZipCode:        () => String(rand(10000, 99999)),

  // Phone
  $randomPhoneNumber:    () => `+1${rand(200,999)}${rand(2000000,9999999)}`,
  $randomPhoneNumberExt: () => `+1${rand(200,999)}${rand(2000000,9999999)} x${rand(100,9999)}`,

  // Lorem
  $randomLoremWord:      () => pick(LOREM_WORDS),
  $randomLoremWords:     () => Array.from({length: rand(3,6)}, ()=>pick(LOREM_WORDS)).join(" "),
  $randomLoremSentence:  () => `${Array.from({length: rand(6,12)}, ()=>pick(LOREM_WORDS)).join(" ")}.`,
  $randomLoremParagraph: () => Array.from({length: rand(3,5)}, ()=>`${Array.from({length: rand(6,12)}, ()=>pick(LOREM_WORDS)).join(" ")}.`).join(" "),

  // Date / time
  $timestamp:            () => String(Math.floor(Date.now() / 1000)),
  $isoTimestamp:         () => new Date().toISOString(),
  $randomDateFuture:     () => new Date(Date.now() + rand(1, 365) * 86400000).toISOString(),
  $randomDatePast:       () => new Date(Date.now() - rand(1, 365) * 86400000).toISOString(),
  $randomDateRecent:     () => new Date(Date.now() - rand(0, 7) * 86400000).toISOString(),
  $randomMonth:          () => pick(MONTHS),
  $randomWeekday:        () => pick(WEEKDAYS),

  // Finance
  $randomBankAccount:    () => String(rand(10000000, 99999999)),
  $randomBankAccountName:() => `${pick(FIRST_NAMES)} ${pick(LAST_NAMES)}`,
  $randomCreditCardMask: () => `xxxx-xxxx-xxxx-${rand(1000,9999)}`,
  $randomCurrencyCode:   () => pick(["USD","EUR","GBP","JPY","CAD","AUD","CHF","CNY","SEK","NOK"]),
  $randomCurrencyName:   () => pick(["US Dollar","Euro","British Pound","Japanese Yen","Canadian Dollar"]),
  $randomCurrencySymbol: () => pick(["$","€","£","¥","₹","₽","₩","₦","₱","฿"]),
  $randomBitcoin:        () => `${rand(1,9)}${Math.random().toString(36).slice(2,33)}`,
  $randomPrice:          () => `${rand(1,999)}.${rand(0,99).toString().padStart(2,"0")}`,

  // Misc
  $randomFileName:       () => `${Math.random().toString(36).slice(2,10)}.${pick(["txt","pdf","jpg","png","csv","json","xml"])}`,
  $randomFileType:       () => pick(["image","video","audio","text","application"]),
  $randomFilePath:       () => `/home/user/${Math.random().toString(36).slice(2,10)}.${pick(["txt","pdf","jpg"])}`,
  $randomMimeType:       () => pick(["application/json","text/plain","image/jpeg","application/pdf","text/html"]),
  $randomSemVer:         () => `${rand(0,9)}.${rand(0,99)}.${rand(0,999)}`,
  $randomAbbreviation:   () => Array.from({length:rand(2,5)}, ()=> "ABCDEFGHIJKLMNOPQRSTUVWXYZ"[rand(0,25)]).join(""),
  $randomWord:           () => pick(LOREM_WORDS),
  $randomWords:          () => Array.from({length: rand(2,5)}, ()=>pick(LOREM_WORDS)).join(" "),
  $randomNoun:           () => pick(["cat","dog","car","tree","book","phone","table","house","chair","window"]),
  $randomVerb:           () => pick(["run","jump","swim","fly","read","write","build","create","delete","update"]),
  $randomAdjective:      () => pick(["quick","lazy","happy","sad","bright","dark","small","large","hot","cold"]),
};

export const DYNAMIC_VAR_NAMES = new Set(Object.keys(DYNAMIC_VARS));

export function isDynamicVar(name: string): boolean {
  return name.startsWith("$");
}

/**
 * Generate a fresh set of dynamic variable values.
 * Call this once per request/curl copy so all occurrences of the same
 * variable get the same value within a single request.
 */
export function resolveDynamicVars(vars: Record<string, string>): Record<string, string> {
  const result = { ...vars };
  for (const [name, gen] of Object.entries(DYNAMIC_VARS)) {
    if (!(name in result)) {
      result[name] = gen();
    }
  }
  return result;
}

/**
 * Stable preview values for the tooltip — one sample per variable.
 * Re-computed once, not on every render.
 */
let _previewCache: Record<string, string> | null = null;
export function getDynamicVarPreviews(): Record<string, string> {
  if (!_previewCache) {
    _previewCache = {};
    for (const [name, gen] of Object.entries(DYNAMIC_VARS)) {
      _previewCache[name] = gen();
    }
  }
  return _previewCache;
}
