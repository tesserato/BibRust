var Tabulator = require('tabulator-tables');

// declare var tabledata:Array<any>


var selectedtags = new Set([]);
var fields = {};

for (var e of tabledata) {
  // console.log(e);
  for (var f in e) {
    // console.log(f);
    if (fields[f]) {
      fields[f] += 1;
    } else {
      fields[f] = 1;
    }
  }
}

const sortedfields = Object.keys(fields).sort(function (a, b) { return fields[b] - fields[a] });
// console.log(sortedfields);

var hcm = [
  {
    label: "Hide this column",
    action: function (e, column) {
      column.hide();
    }
  },
  {
    label: "Show all columns",
    action: function (e, column) {
      for (const col of column.getTable().getColumns()) {
        col.show();
      }
    }
  },
  {
    label: "Delete this column",
    action: function (e, column) {
      const conf = confirm("Really? This can't be undone!");
      if (conf) {
        column.delete();
      }
    }
  },
  {
    label: "Add new column",
    action: function (e, column) {
      var [field, title] = window.prompt("field Title").split(' ');
      table.addColumn({ title: title, field: field }, true, column);
      table.updateColumnDefinition(field, { downloadTitle: field, width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm })
    }
  },
];

function customHeaderFilter(headerValue, rowValue, rowData, filterParams) {
  //headerValue - the value of the header filter element
  //rowValue - the value of the column in this row
  //rowData - the data for the row being filtered
  //filterParams - params object passed to the headerFilterFuncParams property
  if (headerValue && !rowValue) {
    return false;
  }
  else {
    return true;
  }
}
ww = 200;
var types = { values: ["article", "inproceedings", "unpublished", "incollection", "book", "report", "proceedings", "collection", "misc", "online", "thesis"] };

var coldef = [                 //define the table columns
  {
    title: "OK?",
    field: "reviewed",
    titleDownload: "reviewed",
    headerFilter: true,
    hozAlign: "center",
    formatter: "tickCross",
    sorter: "boolean",
    formatterParams: {
      allowEmpty: false,
      allowTruthy: true,
      tickElement: "&starf;",
      crossElement: false,
    },
    cellClick: function (e, cell) {
      cell.setValue(!cell.getValue(), true)
    },
    headerFilterFunc: customHeaderFilter,

  },
  { title: "Type", field: "type", titleDownload: "type", editor: "select", editorParams: types, headerFilter: "input", headerContextMenu: hcm, validator: "required" },
  { title: "Key", field: "key", titleDownload: "key", editor: "input", headerFilter: "input", headerContextMenu: hcm, validator: "required", validator: "unique" },
  { title: "Author", field: "author", titleDownload: "author", width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm },
  { title: "Editor", field: "editor", titleDownload: "editor", width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm },
  { title: "Title", field: "title", titleDownload: "title", width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm },
  { title: "Year", field: "year", titleDownload: "year", editor: "input", headerFilter: "input", headerContextMenu: hcm },
  { title: "Journal", field: "journal", titleDownload: "journal", width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm },
  { title: "url", field: "url", titleDownload: "url", width: ww, editor: "input", headerFilter: "input", 
  formatter: "link",
  formatterParams: {target: "_blank"},
  headerContextMenu: hcm 
},
  { title: "File", field: "file", titleDownload: "file", width: ww, editor: "input", headerFilter: "input", formatter: "link", formatterParams: {target: "_blank"}, headerContextMenu: hcm },
  { title: "Abstract", field: "abstract", titleDownload: "abstract", width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm },
  { title: "Tags", field: "tags", titleDownload: "tags", width: ww / 2, editor: "input", headerFilter: "input", headerContextMenu: hcm },


  // {title:"Task Progress", field:"progress", hozAlign:"left", formatter:"progress", editor:true},
  // {title:"Gender", field:"gender", width:95, editor:"select", editorParams:{values:["male", "female"]}},
  // {title:"Rating", field:"rating", formatter:"star", hozAlign:"center", width:100, editor:true},
  // {title:"Color", field:"col", width:130, editor:"input"},
  // {title:"Date Of Birth", field:"dob", width:130, sorter:"date", hozAlign:"center"},
  // {title:"Driver", field:"car", width:90,  hozAlign:"center", formatter:"tickCross", sorter:"boolean", editor:true},
];

var existingfields = new Set([]);

for (o of coldef) {
  existingfields.add(o.field)
}

for (field of sortedfields) {
  if (!existingfields.has(field)) {
    e = {
      title: field[0].toUpperCase() + field.slice(1),
      field: field,
      titleDownload: field,
      width: ww / 2,
      editor: "input",
      headerFilter: "input",
      headerContextMenu: hcm
    }
    coldef.push(e);
  }
}

function fullscreen() {
  if (elem.requestFullscreen) {
    elem.requestFullscreen();
  } else if (elem.webkitRequestFullscreen) { /* Safari */
    elem.webkitRequestFullscreen();
  } else if (elem.msRequestFullscreen) { /* IE11 */
    elem.msRequestFullscreen();
  }
}

function updatealltags() {
  alltags = new Set([]);
  for (const e of tabledata) {
    if (e['tags']) {
      for (const tag of e['tags'].split(",")) {
        alltags.add(tag);
      }
    }
  }

  alltags = Array.from(alltags).sort();
  // console.log(alltags)

  parentdiv = document.getElementById("tags");
  parentdiv.innerHTML = "";
  for (const tag of alltags) {
    div = document.createElement("div");
    txt = document.createTextNode(tag);
    div.appendChild(txt);
    div.classList.add("tag");
    if (selectedtags.has(tag)){
      div.classList.add("selectedtag");
    } else {
      div.classList.add("deselectedtag");
    }
    div.onclick = handleLeftClick;
    div.oncontextmenu = handleRightCLick;
    parentdiv.appendChild(div);
  }
  var btn = document.createElement("BUTTON");   // Create a <button> element
  btn.innerHTML = "Clear tags";                   // Insert text
  btn.classList.add("btn");
  btn.onclick = function (event) {
    selectedtags.clear();
    table.clearFilter();
    for (e of document.getElementsByClassName("tag")) {
      e.classList.remove("selectedtag");
      e.classList.add("deselectedtag");
    }
  }
  parentdiv.appendChild(btn);

  var btn = document.createElement("BUTTON");   // Create a <button> element
  btn.innerHTML = "Fullscreen";                   // Insert text
  btn.classList.add("btn");
  btn.onclick = function (event) {
    var elem = document.documentElement;
    if (elem.requestFullscreen) {
      elem.requestFullscreen();
    } else if (elem.webkitRequestFullscreen) { /* Safari */
      elem.webkitRequestFullscreen();
    } else if (elem.msRequestFullscreen) { /* IE11 */
      elem.msRequestFullscreen();
    }
  }
  parentdiv.appendChild(btn);

  var btn = document.createElement("BUTTON");   // Create a <button> element
  btn.innerHTML = "Download as .csv";                   // Insert text
  btn.classList.add("btn");
  btn.onclick = function (event) {
    // var elem = document.documentElement;
    table.download("csv", "bibliography.csv", { bom: true });
  }
  parentdiv.appendChild(btn);

  var btn = document.createElement("BUTTON");   // Create a <button> element
  btn.innerHTML = "Show all columns";                   // Insert text
  btn.classList.add("btn");
  btn.onclick = function (event) {
    for (col of table.getColumns()) {
      col.show();
    }
  }
  parentdiv.appendChild(btn);

  // var btn = document.createElement("BUTTON");   // Create a <button> element
  // btn.innerHTML = "Save JSON";                   // Insert text
  // btn.classList.add("btn");
  // btn.onclick = function (event) {
  //   table.download("json", "bibliography.json");
  // };
  // parentdiv.appendChild(btn);
}

var table = new Tabulator("#table", {
  // keybindings:{
  //   "undo" : "ctrl + z",
  //   "redo" : "ctrl + y",
  // },
  cellEdited: function (cell) {
    if (cell.getColumn().getField() == "tags") {
      updatealltags();
    }
  },
  rowContextMenu: [
    {
      label: "Toggle freeze row",
      action: function (e, row) {
        if (row.isFrozen()) {
          row.unfreeze();
        } else {
          row.freeze()
        }
      }
    },
    {
      label: "Insert row above",
      action: function (e, row) {
        table.addData([{}], true, row.getElement());
      }
    },
    {
      label: "Insert row below",
      action: function (e, row) {
        table.addData([{}], false, row.getElement());
      }
    },
    {
      label: "Delete Row",
      action: function (e, row) {
        row.delete();
      }
    },
    {
      label: "Download data as .csv",
      action: function (e, row) {
        table.download("csv", "bibliography.csv", { bom: true });
      }
    },
  ],
  // selectable:true,
  persistenceID:"bibtable",
  persistence:{
    sort:true, //Enable sort persistence
    filter:true, //Enable filter persistence
    columns:true //Enable column layout persistence
  },
  data: tabledata,           //load row data from array
  downloadRowRange: "all",
  layout: "fitDataFill",      //fit columns to width of table
  // responsiveLayout:"hide",  //hide columns that dont fit on the table
  tooltips: true,            //show tool tips on cells
  addRowPos: "top",          //when adding a new row, add it to the top of the table
  history: true,             //allow undo and redo actions on the table

  pagination: "local",       //paginate the data
  paginationSize: 100,         //allow 100 rows per page of data

  movableColumns: true,      //allow column order to be changed
  resizableRows: true,       //allow row order to be changed
  // autoColumns:true,
  // initialSort: [             //set the initial sort order of the data
  //   { column: "name", dir: "asc" },
  // ],
  columns: coldef,
});

updatealltags(table.getColumn("tags"));

function customFilter(data, selectedtags) {
  if (!data.tags) {
    return false;
  }
  else {
    let tags = new Set(data.tags.split(","));
    for (let elem of selectedtags) {
      if (!tags.has(elem)) {
        return false;
      }
    }
    return true;
  }
}

function handleLeftClick(event) {  
  let tag = event.target.textContent;
  if (selectedtags.has(tag)) {
    selectedtags.delete(tag);
    event.target.classList.remove("selectedtag");
    event.target.classList.add("deselectedtag");
    if (selectedtags.length == 0) {
      table.clearFilter();
    }
    else {
      table.setFilter(customFilter, selectedtags);
    }
    // table.update();
  }
  else {
    selectedtags.add(tag);
    event.target.classList.remove("deselectedtag");
    event.target.classList.add("selectedtag");
    table.setFilter(customFilter, selectedtags);
    // table.update();
  }
}

function handleRightCLick(event) {
  event.preventDefault()
  // console.log(event)
  var oldtag = event.target.textContent;
  var newtag = prompt("Rename tag ( Warning: Can't be undone! )", oldtag);
  // console.log(newtag)
  if (newtag != null) {
    console.log("renaming", oldtag, " to ", newtag);
    // console.log(table.getColumn("tags").getCells());
    for (c of table.getColumn("tags").getCells()) {
      // console.log(c.getValue())
      if (c.getValue()) {
        c.setValue(c.getValue().replace(oldtag, newtag).split(',').map(e => e.trim()).filter(e => e).join(','));
      }
    }
    if (selectedtags.has(oldtag)){
      selectedtags.delete(oldtag);
      selectedtags.add(newtag);
    }
    updatealltags()
  }
}

document.addEventListener('keydown', hadleKey);
function hadleKey(e) {
  // console.log(e);
  if (e.ctrlKey) {
    switch (e.key) {
      case "z":
        // console.log("undo");
        table.undo();
        break;
      case "y":
        // console.log("redo");
        table.redo();
        break;
      case "s":
        // console.log("redo");
        e.preventDefault();
        table.download("csv", "bibliography.csv", { bom: true });
        break;
      default:
      // code block
    }
  }
}

document.body.appendChild(document.getElementsByClassName("tabulator-footer")[0]);

document.getElementById("splash").remove();
