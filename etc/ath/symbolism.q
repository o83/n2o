//Namespace for all tables.
namespace:"symbolism";
//Set timer to update last trading date.
.z.ts: {ltr::lastTradingDate .z.d};
system "t 1000";
//Wraps tablename with namespace.
//@param table name
//@return wrapped string
tname:{`$".",namespace,".",string x};
//Set table attributes to group in key columns.
//@param table
//@return attributed table
sattr:{c:keys x;n:count c;n!@[;c;`g#]0!x};
//Set table's attributes inplace.
//@param table
//@return ::
xsattr:{x::sattr x;};
//Sync table to hard drive.
//@param tablename
//@return tablename
savetable:{xsattr value tname x;(hsym `$ namespace,"/",(string x)) set value tname x};
//Sync all tables in namespace.
//@param ::
//@return names list
savetbls:{t:system "v .",namespace;savetable'[t]};
//Loads table from hard into namespace.
//@param tablename
//@return tablename
loadtable:{value ".",namespace,".",x," :get `:symbolism/",x};
//Loads all tables stored from namespace.
//@param ::
//@return list of tablenames
restore:{loadtable'[system "ls symbolism/"]};
//Upsert with emiting of event to subscribed clients.
//@param tablename
//@param table
tupsert:{upsert[tname x;y];emit[x;y]};
//Clear table with emiting of event to subscribed clients.
tclear:{delete from tname x;emit[`Clr;x];};
//Find duplicates in table (service function).
//@param table
//@return table
dups:{select from x where 1<(count;i) fby ticker};
restore[];
//Logging of incoming connections
conlogs:([]date:"D"$();time:"T"$();ip:`$();user:`$();action:`$());
clog:{`conlogs insert (.z.d;.z.t;`$ addrp .z.a;.z.u;x);};
/User access permissions
.perm.users:([user:`$()] password:());
.perm.toString:{[x] $[10h=abs type x;x;string x]};
.perm.encrypt:{[u;p] md5 raze .perm.toString p,u};
.perm.add:{[u;p] `.perm.users upsert (u;.perm.encrypt[u;p]);};
.perm.isSU:{[u] u in exec user from .perm.users};
.perm.chkUser:{[u;p] $[not .perm.isSU[u];1b;$[.perm.encrypt[u;p]~.perm.users[u][`password];1b;0b]]};
.perm.isSproc: {al:(system "f "),(system "v ");$[10h=type x;(`$ string(*:)parse x)in al;(`$ string(*:)x)in al]};
.perm.add[`root;`Uncle0n];
.perm.readOnly:{res:first {[q;exe] $[exe;@[value;q;{(`error;x)}]; ()]}[x;] peach 10b;
    if[(2=count res) and `error~first res;$[last[res]~"noupdate";'last res;'"permissions"]];res};
.perm.execSuQuery: {value x};
.perm.execUserQuery: {$[.perm.isSproc x;value x;.perm.readOnly x]};
.z.pw:{[user;pwd] .perm.chkUser[user;pwd]};
.z.pg:{$[.perm.isSU .z.u;.perm.execSuQuery x;.perm.execUserQuery x]};
//Clients handlers for async events.
hds:([hd:`int$()];ip:`int$();usr:`symbol$());
//Set callback on client opens connection.
.z.po:{c:((count cols hds)-3)#`;`hds upsert raze (x;.z.a;.z.u;c);clog `connect;};
//Set callback on client closes connection.
.z.pc:{![`hds;enlist(=;`hd;x);0b;`symbol$()];clog `disconnect;};
//Subscribe on pecific event.
//@param event - symbol;callback - symbol
//@raturn ::
subsc:{[ev;cl] if[not ev in cols hds;![`hds;();0b;(enlist ev)!enlist enlist `]];![`hds;enlist(=;`hd;.z.w);0b;(enlist ev)!enlist enlist cl];};
//Unsubscribe from specific event.
//@param event - symbol
unsub:{![`hds;enlist(=;`hd;.z.w);0b;(enlist x)!enlist enlist `];};
//Raize specific event.
//@param event - symbol
//@param data - table
//@return ::
emit:{[ev;ar] if[not ev in cols hds;:0N];t:?[hds;enlist(<>;ev;enlist `);0b;()];if[0~count t;:0N];{neg[z[`hd]](z[x];y)}[ev;ar;]'[() xkey t];};
//Converts ticker to bbgid.
//@param ticker - symbol
//@return bbgid - symbol
bbgActT:{(*:)?[`date xdesc .symbolism.EquityTickers;enlist(=;`ticker;`x);();`bbgid]};
//Converts tickers to bbgids.
//@param tickers - list of symbols
//@return bbgids - list of symbols
bbgsActT:{bbgActT'[x]};
//Converts ticker to bbgid.
//@param ticker - symbol
//@param date - date
//@return bbgid - symbol
bbgTD:{[t;d](*:)?[.symbolism.EquityTickers;((=;`ticker;`t);(>=;`date;`d));();`bbgid]};
//Converts tickers to bbgids.
//@param tickers - list of symbols
//@param dates - dates list
//@return bbgids - list of symbols
bbgsTD:{bbgTD'[x;y]};
//Retrieve information about stock by its bbgid.
//@param bbgid - symbol
//@return table
bbgInfo:{?[.symbolism.Tickers;enlist(=;`bbgid;`x);0b;()]};
//Retrieve fundamentals data for stock.
//@param bbgid - symbol
//@return table
bbgFund:{?[.symbolism.EquityFundamentals;enlist(=;`bbgid;`x);0b;()]};
//Retrieve stock names history.
//@param bbgid - symbol
//@reutrn table
bbgHist:{?[.symbolism.EquityTickers;enlist(=;`bbgid;`x);0b;()]};
//Retrieve ticker name by bbgid,date.
//@param bbgid - symbol
//@param date - date
//@return ticker - symbol
tickBD:{[b;d](*:)?[.symbolism.EquityTickers;((=;`bbgid;`b);(>=;`date;`d));();`ticker]};
//Retrieve latest stock name by bbgid.
//@param bbgid - symbol
//@return ticker - symbol
tickActB:{(*:)?[desc .symbolism.EquityTickers;enlist(=;`bbgid;`x);();`ticker]};
//Retrieve latest stocks names by list of bbgids.
//@param bbgids - list of symbols
//@return tickers - list of symbols
ticksActB:{tickActB'[x]};
//Add `ticker column to input table contains `bbgid column.
//@param table
//@return table
fillTick:{x lj 1!select bbgid,ticker from ticksLast[]};
//Retrieve active tickers.
//@param ::
//@return table
ticksAct:{?[.symbolism.EquityTickers;enlist(=;`date;(max;`date));0b;()]};
//Retrieve latest ticker names.
//@param ::
//@return table
ticksLast:{select from .symbolism.EquityTickers where 0=(rank;neg date) fby bbgid};
//Converts bbgid to bbg composite.
//@param bbgid - symbol
//@return bbgid - symbol
bbg2bbcomp:{[b] tc:select from .symbolism.Tickers where bbgid=b;if[0=count tc;:`];bc:first exec bbcomp from tc;if[[bc<>`];:bc];
   bc:first exec bbgid from .symbolism.Tickers where bbgid=b,pricing_source=`US;$[[bc<>`];bc;b]};
//Converts bbgids to bbg composites.
//@param bbgids - list of symbols
//@return bbgids - list of symbols
bbgs2bbcomp:{bbg2bbcomp'[x]};
//Converts comstock ticker to bbg composite.
//@param ticker - symbol
//@return bbgid - symbol
com2bbcomp:{exec (*:) bbcomp from .symbolism.Tickers where ticker=x};
//Converts com tickers to bbg composites.
//@param coms - list of symbols
//@return bbgids - list of symbols
coms2bbcomp:{com2bbcomp'[x]};
//Retrieve all bbgids which are ADR's.
//@param ::
//@return table
adrs:{1!select bbgid,adr:{`ADR=x}sec_type from .symbolism.Tickers where bbgid in x,0=(rank;id_bb_unique) fby bbgid};
//Get number of day of week.
//@param ::
//@return day - int
dayOfWeek:{6 7 1 2 3 4 5 x mod 7};
//Check if date is holiday.
//@param date - date
//@return bool
isHoliday:{hl:?[.symbolism.HolidaysCalendar;enlist(=;`date;x);0b;enlist[`status]!enlist `status][`status][0];$[[hl=`Closed];1b;0b]};
//Check if date is trading one.
//@param date - date
//@return bool
isTradingDate:{$[dayOfWeek[x]in 6 7;0b;$[isHoliday[x];0b;1b]]};
//Get plain list of trading dates (excludes holidays and non-working days).
//@param date from
//@param date to
//@return list of dates
trdays:{desc d where isTradingDate'[d:x+til 1+y-x]};
//Retrieve last work day before specified one.
//@param date - date
//@return date - date
lastWorkDate:{x-(3 1 1 1 1 1 2)[-1+dayOfWeek x]};
//Retrieve last trading date before specified one.
//@param date
//@return date
lastTradingDate:{l:lastWorkDate[x];$[isHoliday[l];.z.s l;l]};
//Converts date to int representation (needs for BloombergDB API).
//@param date
//@return int
date2int:{-6h$(ssr[string x;".";""])};
//Casts int/long to float.
//@param value
//@return float
toFloat:{[c;p]"f"$(("i"$(c*xexp[10;p]))%xexp[10;p])};
//An arbitration function to use in ohlc.
//arb:{c:{count each group x}x;c[0]:0;c?max c};
arb:{c:{count each group x}x;c[0]:0;c:where c=(count c)#max c;c m?min m:abs c-d:dev x};
//Retrieve open,high,low,close,volume table with arbitration.
//@param ::
//@return table
ohlc:{(0!select src:`arb,open:arb open,high:arb high,low:arb low,close:arb close,
    volume: arb volume by bbgid,date from .symbolism.TradingRecords where 
    date<.z.d,bbgid in ta[`bbgid]) lj 1!ta:select bbgid,ticker,sym_id:"h"$ i from ticksAct[]};
//Arbitrate adjustments
//@param ::
//@return table
adjustmentsArb:{a:update pt:{(*:)where {y in x}'[(regular;stock;spinoff);x]}'[payment_type] from
   (2!0!update amount:sum amount by bbgid,date,src from .symbolism.Adjustments where payment_type in regular,src=`bloombergdb);
   a:update amount:{c:{count each group x}x;c?max c}amount by bbgid,date from a;
   select bbgid,date,pt,amount from (update dfl:({$[x=0Nd;y;$[10>abs y-z;x;y]]}\)[0Nd;date;prev date] by 
   bbgid,pt from `date xdesc a) where bbgid<>`,0=(rank;neg feeddate) fby ([]bbgid;dfl;pt)};
//Retrieve open,high,low,close,volume table with arbitration, with dates scope.
//@param date from
//@param date to
//@return table
ohlcPfDD:{[df;dt]update pf:(prds {1-x*10000f%y}[({y}':)dvd;close])%prds({y}':)spl by bbgid from `date xdesc select from stpj where date within(df;dt)};
//Retrieve shares information limited by dates range.
//@param date from
//@param date to
//@return table
sharesDD:{[df;dt]select from .symbolism.Shares where date within(df;dt),0=({c:(count each group x);c[0]:0;rank neg c x};shares_out) fby ([]bbgid;date)};
//Retrieve shares information limited by dates range, bbgid.
//@param date from
//@param date to
//@return table
sharesBDD:{[bbg;df;dt]select from .symbolism.Shares where date within(df;dt),bbgid=bbg,0=({c:(count each group x);c[0]:0;rank neg c x};shares_out) fby ([]bbgid;date)};
//Retrieve earnings for given period.
//@param date from
//@param date to
//@return table
earningsDD:{[df;dt]s:0!select by bbgid,date from .symbolism.Earnings where date within(df;dt);select bbgid,date,src,time,amount,status,quarter,year,feeddate from
    (update dfl:({$[x=0Nd;y;$[30>abs y-z;x;y]]}\)[0Nd;date;prev date] by bbgid from s) where bbgid<>`,0=(rank;neg feeddate) fby ([]bbgid;dfl)};
//Retrieve adjustment data (dividends,splits,spinoffs) for given period.
//@param date from
//@param date to
//@return table
dividendsDD:{[df;dt]s:0!select by bbgid,date from (update amount:sum amount by bbgid,date,src from .symbolism.Adjustments where payment_type in regular,src=`bloombergdb) where date within(df;dt);
    select bbgid,date,src,payment_type,declaration_date,record_date,payment_date,amount,frequency,split_from,split_to,feeddate from
    (update dfl:({$[x=0Nd;y;$[10>abs y-z;x;y]]}\)[0Nd;date;prev date] by bbgid from s) where bbgid<>`,0=(rank;neg feeddate) fby ([]bbgid;dfl)};
//Retrieve confcalls for given period.
//@param date from
//@param date to
//@return table
confcallsDD:{[df;dt]select by bbgid,call_date from .symbolism.ConfCalls where call_date within(df;dt)};
//Retrieve open,high,low,close,volume table with arbitration, with dates scope for OTC tickers.
//@param date from
//@param date to
//@return table
otcOhlcDD:{[df;dt]3!0!select src:`arb,open:arb open,high:arb high,low:arb low,close:arb close,volume: arb volume by bbgid,date 
    from .symbolism.OtcTradingRecords where date within(df;dt)};
//Retrieve OTC tickers has to be included into fulactive list by specific criteria.
//@param ::
//@return table
otcFat:{exec distinct bbgid from .symbolism.OtcTradingRecords where date>=ltr-14,(50000*10000)<({avg x};close*volume) fby bbgid};
//Converts bbgid to bbg composite for OTC ticker.
//@param bbgid - symbol
//@return bbgid - symbol
otcBbg2bbcomp:{first exec bbcomp from .symbolism.Tickers where (ticker=x),(pricing_source in `PQ`UU`UV)};
//Converts bbgids to bbg composites for OTC tickers.
//@param bbgids - list of symbols
//@return bbgids - list of symbols
otcBbgs2bbcomp:{otcBbg2bbcomp'[x]};
//Adjustments type which must be interpreted as dividend.
regular:(`$"Cash";`$"Regular Cash";`$"Special Cash";`$"Income";`$"Partnership Dist";`$"Short Term Cap Gain";`$"Rights Issue";`$"Interest on Capital";`$"Pro Rata";`);
//Adjustments type must be interpreted as spinoff.
spinoff:(`$"Spinoff");
//Adjustments type must be interpreted as split.
stock:(`$"Stock Split";`$"Stock Dividend");
//Calculates adjustment coefficients for shares out and volumes adjustments.
//@param date
//@param date
//@return table
adjcoefsvDD:{[df;dt]update adj_div:1f^({(y)}':)(*\)adj_div by bbgid from update adj_div:{$[x=`$"Stock Split";z*y;$[x=`$"Spinoff";z%y;z]]}'[payment_type;amount;adj_div]
    by bbgid from select from stpj where date within(df;dt)};
//Calculates adjustment coefficients for shares out and volumes adjustments specified by bbgid's.
//@param bbgid's list
//@param date
//@param date
//@return table
adjcoefsvBDD:{[bb;df;dt]update adj_div:1f^({(y)}':)(*\)adj_div by bbgid from update adj_div:{$[x=`$"Stock Split";z*y;$[x=`$"Spinoff";z%y;z]]}'[payment_type;amount;adj_div]
    by bbgid from select from stpj where bbgid in bb,date within(df;dt)};
//Updates persistent ids table with specified ticker (if doesn't exist).
//@param ticker
//@return ::
upersist:{u:x[`ticker] except .symbolism.PersistIds[`ticker];u:`ticker xasc select ticker,date:.z.d from x where ticker in u;
    l:1+max .symbolism.PersistIds[`sym_id];`.symbolism.PersistIds upsert select ticker,sym_id:"h"$i+l,date from u;}
//Converts ticker from CQS symbology to Comstock one.
//@param suffixtable - symbol
//@param ticker - symbol
//@return ticker - symbol
cqs2com:{[suff;sym] if[0=count suff;:sym];s:string sym;f:suff[`cqs]0;t:suff[`com]0;r:ssr[s;f;t];$[s~r;.z.s[1_suff;sym];`$ r]};
//Converts ticker from CMS symbology to Comstock one.
//@param suffixtable - symbol
//@param ticker - symbol
//@return ticker - symbol
cms2com:{[suff;sym] if[0=count suff;:sym];s:string sym;f:" ",suff[`cms]0;t:suff[`com]0;r:ssr[s;f;t];$[s~r;.z.s[1_suff;sym];`$ r]};
//Converts ticker from NYSE TAQ symbology to Comstock one.
//@param suffixtable - symbol
//@param ticker - symbol
//@return ticker - symbol
taq2com:{[suff;sym] if[0=count suff;:sym];s:string sym;f:".",suff[`cms]0;t:suff[`com]0;r:ssr[s;f;t];$[s~r;.z.s[1_suff;sym];`$ r]};
//Converts ticker from CSI symbology to Comstock one.
//@param suffixtable - symbol
//@param ticker - symbol
//@return ticker - symbol
csi2com:{[suff;sym] if[0=count suff;:sym];s:string sym; `$ $[0<count ss[s;"-W"];ssr[s;"-W";"+"];ssr[s;"+";"-"]]};
//Converts ticker from specified symbology to Comstock one.
//@param symbology - symbol
//@param ticker - symbol
//@return ticker - symbol
symconv:{[stype;sym] (value ((string stype),"2com"))[select[> count each .symbolism.Suffixes[stype]] from .symbolism.Suffixes;sym]};
//Converts list of tickers from specified symbology to Comstock one.
//@param symbology - symbol
//@param tickers - list of symbols
//@return tickers - list of symbols
symsconv:{[stype;syms] symconv[stype;]'[syms]};
//Try to find missed bbgids for specified tickers automatically.
//@param tickers - list of symbols
//@return table
recoverSyms:{f:select ticker,bbgid from .symbolism.Tickers where (ticker in x),(pricing_source=`US);nf:x except (exec ticker from f);sf:nf where nf like "*-*";
    fsf:raze {sp: "-" vs string x;n: first sp;s: ssr[last sp;"*";""];select ticker:x,bbgid from .symbolism.Tickers where (ticker like n," * ",s),(pricing_source=`US)}'[sf];f,fsf};
//Find missed bbgids for specified src.
//@param src - symbol
//@param datefrom - date
//@param dateto - date
//@return table
srcMissDD:{[s;df;dt]fillTick key select by bbgid,date from .symbolism.TradingRecords where date within(df;dt),1b=({(count y)#(not x in y)and 2<count y}[s;];src) fby ([]bbgid;date)};
//Fills specified table with Tape and Exchange columns.
//@param table
//@return table
getTapeExchCms:{[syms;dt]t:{`sym`ticker!(x;symconv[`cms;x])}'[syms];f:update tape:{if[x=`;:`];if[x=`N;:`A];if[x=`Q;:`C];`B}'[exchange] from
    select ticker,exchange from (`date xasc .symbolism.FullActive) where ticker in t[`ticker],({y=y[y bin x]}[dt;];date) fby ticker;
    f:f lj (`ticker xkey t);ms:select from t where not ticker in f[`ticker];r:{e:first exec eqy_prim_exch from `date xdesc .symbolism.EquityFundamentals where bbgid=bbgActT x[`ticker];
    `sym`ticker`exchange!(x[`sym];x[`ticker];$[0h<>type e;e;`])}'[ms];if[0h=type r;:select sym,ticker,exchange,tape from f];
    r:update tape:{if[x=`;:`];if[x like "New York";:`A];if[x like "NASDAQ*";:`C];`B}'[exchange] from r;
    r: update exchange:{if[x like "New York";:`N];if[x like "NASDAQ*";:`Q];`N}'[exchange] from r;(select sym,ticker,exchange,tape from f),r};
//Assembles fullactive table.
//@param ::
//@return table
fullactive:{fa:(`bbgid xkey select from .symbolism.FullActive where date=.z.d) lj (`bbgid xkey ohlcPfDD[ltr;ltr]);sh:select bbgid,shares_out from sharesDD[ltr;ltr];fa:fa lj (`bbgid xkey sh);
    fa:select ticker,instrument_type:`E,exchange,price_digits:4,contract_multiplier:1,etf,round_lot_size,test_issue,adr,close:0f^toFloat[close%10000;2],
    volume:0j^volume,shares_out:0j^shares_out from fa;fo:(`bbgid xkey select from .symbolism.OtcFullActive where date=.z.d,((bbgid in otcFat[]) or (ticker in `NBFT`PRED))) lj (`bbgid xkey otcOhlcDD[ltr;ltr]);
    fo:select ticker,instrument_type:`E,exchange,price_digits:3,contract_multiplier:1,etf,round_lot_size,test_issue,adr,close:0f^toFloat[close%10000;2],
    volume:0j^volume,shares_out:0j from fo;`ticker xasc fa,(select from fo where not ticker in fa[`ticker])};
//Updates cache for fast adjustments.
upstpj:{st: (ohlc[] lj 2!0!.symbolism.Shares) lj 2!adjustmentsArb[];
    st:update dvd:amount from st where pt=0;
    st:update spl:1%amount from st where pt=2;
    st:update spl:amount from st where pt=1;
    stpj::sattr 2!select bbgid,date,open:0^open,high:0^high,low:0^low,
    close:0^close,volume:0^volume,pt,amount:0f^amount,dvd,spl,
    ticker,sym_id,float_shares:0^float_shares,shares_out:0^shares_out from st;};
//Special for mathlab (will be removed later)
ohlcAdjDD:{[df;dt] select bbgid,date,open,high,low,close,adj_open:open*pf,adj_close:close*pf,volume from ohlcPfDD[df;dt]};
//Mapping nasdaq exchange codes to bloomberg exchanges
mapExch:{flip `exchange`name!(`N`A`P`Z`Q`G`S;(`$"New York";`$"NYSE MKT LLC";`$"NYSE Arca";`$"BATS";`$"NASDAQ GS";`$"NASDAQ GM";`$"NASDAQ CM"))};
//Retrieves self address and port.
//@param ::
//@return string
addrp:{("."sv string"i"$0x0 vs .z.a),":",string system "p "};
//Retrieves percent difference between 7-day previous data 
//by sources from TradingRecords compare to new one. Used to
//detect if fullactive is not filled by all sources.
//@param percent deviation - float
//@return table
ohlc7dev:{dts:1_8#desc exec distinct date from .symbolism.TradingRecords;
    avg7d:select avg avg_7d_records by src from (select avg_7d_records:count i by 
    src,date from .symbolism.TradingRecords where date in dts);
    now:select records_last:count i by src from .symbolism.TradingRecords where 
    date = ltr; select from (update rdev:{100*(abs x-y)%x}'[avg_7d_records;records_last] 
    from (avg7d lj now)) where rdev>x,records_last<avg_7d_records};
//Load slack lib for sending messages.
slmsg:`libslack 2:(`slmsg;4);
//Sends message to Slack.
snotify:{[l;m] slmsg[`$"Event from ",addrp[];`$namespace;l;m]};
//Application exit callback.
.z.exit:{savetbls[];snotify[2;`$"Server was terminated."]};


//Add tick_size column to table. Used for BloombergDB requests.
//@param table with fields bbgid,date
//@return table
tickSize:{dSet:`bbgid`date xkey x; mSet:select bbgid,date,pilot,tick_size:{$[x in `G1`G2`G3;5f;1f]}'[pilot] from dSet lj
 (`bbgid`date xkey update date:{lastTradingDate x}'[date] from (select bbgid,date,pilot:pilot_group from .symbolism.PilotGroup));update pilot_group:pilot from (dSet lj (`bbgid`date xkey mSet))};
//Return tick_size, pilot_group data within date for each symbol in FullActive table. Used for BloombergDB requests.
//@param table with fields bbgid,date,sym_id,tick_size, pilot_group
//@return table
TickSizeArr:{[a;b] trdate:trdays [a+1;b+1]; update date:{lastTradingDate x}'[date], tick_size:{$[x in `G1`G2`G3;5f;1f]}'[pilot_group] from (`bbgid`date xkey ((1!select bbgid,ticker,date from .symbolism.FullActive where date in trdate)
   lj (1!select bbgid,ticker,sym_id:"h"$i from ticksAct[]))) lj (2!select bbgid,date, ticker, pilot_group from .symbolism.PilotGroup where date in trdate)};
//Return tick_size, pilot_group data within date for each symbol in FullActive table. Used for BloombergDB requests.
//@param table with fields bbgid,date,sym_id,tick_size, pilot_group
//@return table
tickPilotAct:{[a;b] trdate:trdays [a;b]; select ticker, bbgid from (select [>date] by ticker from .symbolism.FullActive where date in trdate)};
//Return tick_size, pilot_group data within date for each symbol in FullActive table. Used for BloombergDB requests.
//@param table with fields bbgid,date,sym_id,tick_size, pilot_group
//@return table
tickSizeArr:{[a;b] trdate:trdays [a;b];ticks:select distinct ticker from .symbolism.FullActive where date in trdate;arr:(`ticker`date xkey((`ticker xkey ungroup ([] ticker:ticks`ticker;date:count[ticks]#enlist trdate))
lj (1!select ticker,sym_id:"h"$i,bbgid from tickPilotAct[a;b]))) lj (2!select ticker,date, pilot_group from .symbolism.PilotGroup where date in trdate);
update date:{lastTradingDate x}'[date], tick_size:{$[x in `G1`G2`G3;5f;1f]}'[pilot_group]  from arr};
//Return table selected from .symbolism.TradingRecords within range of dates
//@param startdate,enddate
//@return table 
ohlcDD:{[df;dt]3!0!select src:`arb,open:arb open,high:arb high,low:arb low,close:arb close,volume: arb volume by bbgid,date from .symbolism.TradingRecords 
    where date within(df;dt)};


upstpj[];

snotify[3;`$"Server is up."];