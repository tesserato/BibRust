<svelte:head>
	<link href="https://unpkg.com/tabulator-tables@4.6.2/dist/css/tabulator.min.css" rel="stylesheet">
</svelte:head>

<script>
let tabledata = [
  {key: `1989_harvey`,type: `book`,author: `D. Harvey`,abstract: `Seeks to explain the meaning of postmodernism within the contexts of architecture, art, culture, and society, and to identify how accurate and useful it is as a description of contemporary experiences. In the course of this investigation, the author provides a social and semantic history, from the Enlightenment to the present, of modernism and its expression in political and social ideas and movements, with particular reference to changes in the meaning and perception of time and space through history, and how this variance affects individual values and social processes. This publication contains four major sections, namely: the passage from modernity to postmodernity in contemporary culture; the political-economic transformation of late twentieth-century capitalism; the experience of space and time; and the condition of postmodernity. -after Publisher`,booktitle: `The condition of postmodernity: an enquiry into the origins of cultural change`,doi: `10.2307/2072256`,isbn: `0631162925`,issn: `00943061`,title: `The condition of postmodernity: an enquiry into the origins of cultural change`,year: `1989`,file: `C:/Users/tesse/Desktop/Files/Dropbox/BIBrep/0 New BOOKS/1989 The condition of postmodernity an enquiry into the origins of cultural change - Harvey.pdf`},
  {key: `1989_hanson_etal`,type: `inproceedings`,author: `Stephen Jos'e Hanson and Lorien Y Pratt`,booktitle: `Advances in neural information processing systems`,pages: `177--185`,title: `Comparing biases for minimal network construction with back-propagation`,year: `1989`,tags: `#corrupted author`},
  {key: `1989_hanson_etal`,type: `inproceedings`,author: `Stephen Jos'e Hanson and Lorien Y Pratt`,__markedentry: `[tesse:1]`,booktitle: `Advances in neural information processing systems`,pages: `177--185`,title: `Comparing biases for minimal network construction with back-propagation`,year: `1989`},];

import Tabulator from 'tabulator-tables';
// import tabledata from "./result.js";



var hcm = [
  {
    label:"Hide this column",
    action:function(e, column){
        column.hide();
    }
  },
  {
    label:"Show all columns",
    action:function(e, column){
      for (col of column.getTable().getColumns()){
        col.show();
      }
    }
  },

];

ww = 400;
var coldef = [                 //define the table columns
  {title:"OK?", field:"reviewed", downloadTitle:"reviewed", headerFilter:true, hozAlign:"center", formatter:"tickCross", sorter:"boolean", editor:true},
  {title:"Type", field:"type", downloadTitle:"type", editor:"input", headerFilter:"input", headerContextMenu:hcm},
  {title:"Key", field:"key", downloadTitle:"key", editor:"input", headerFilter:"input", headerContextMenu:hcm},
  {title:"Author", field:"author", downloadTitle:"author", width:ww, editor:"input", headerFilter:"input", headerContextMenu:hcm},
  {title:"Editor", field:"editor", downloadTitle:"editor", width:ww, editor:"input", headerFilter:"input", headerContextMenu:hcm},
  {title:"Title", field:"title", downloadTitle:"title", width:ww, editor:"input", headerFilter:"input", headerContextMenu:hcm},
  {title:"Year", field:"year", downloadTitle:"year", editor:"input", headerFilter:"input", headerContextMenu:hcm},
  {title:"Journal", field:"journal", downloadTitle:"journal", width:ww, editor:"input", headerFilter:"input", headerContextMenu:hcm},
  {title:"url", field:"url", downloadTitle:"url", width:ww, editor:"input", headerFilter:"input",formatter:"link", headerContextMenu:hcm},
  {title:"File", field:"file", downloadTitle:"file", width:ww, editor:"input", headerFilter:"input",formatter:"link", headerContextMenu:hcm},
  {title:"Abstract", field:"abstract", downloadTitle:"abstract", width:ww, editor:"input", headerFilter:"input",formatter:"text", headerContextMenu:hcm},
  {title:"Tags", field:"tags", downloadTitle:"tags", width:ww/2, editor:"input", headerFilter:"input", headerContextMenu:hcm},


  // {title:"Task Progress", field:"progress", hozAlign:"left", formatter:"progress", editor:true},
  // {title:"Gender", field:"gender", width:95, editor:"select", editorParams:{values:["male", "female"]}},
  // {title:"Rating", field:"rating", formatter:"star", hozAlign:"center", width:100, editor:true},
  // {title:"Color", field:"col", width:130, editor:"input"},
  // {title:"Date Of Birth", field:"dob", width:130, sorter:"date", hozAlign:"center"},
  // {title:"Driver", field:"car", width:90,  hozAlign:"center", formatter:"tickCross", sorter:"boolean", editor:true},
];


export let table = new Tabulator("#table", {
  rowContextMenu:[
    {
      label:"Toggle freeze row",
      action:function(e, row){
        if (row.isFrozen()){
          row.unfreeze();
        } else {
          row.freeze()
        }
      }
    },
    {
      label:"Insert row above",
      action:function(e, row){
        table.addData([{}], true, row.getElement());
      }
    },
    {
      label:"Insert row below",
      action:function(e, row){
        table.addData([{}], false, row.getElement());
      }
    },
    {
      label:"Delete Row",
      action:function(e, row){
          row.delete();
      }
    },
    {
      label:"Download data as .csv",
      action:function(e, row){
          table.download("csv", "data.csv", {bom:true});
      }
    },
  ],

  data:tabledata,           //load row data from array
  downloadRowRange:"all",
  layout:"fitDataFill",      //fit columns to width of table
  // responsiveLayout:"hide",  //hide columns that dont fit on the table
  tooltips:true,            //show tool tips on cells
  addRowPos:"top",          //when adding a new row, add it to the top of the table
  history:true,             //allow undo and redo actions on the table

  pagination:"local",       //paginate the data
  paginationSize:100,         //allow 7 rows per page of data

  movableColumns:true,      //allow column order to be changed
  resizableRows:true,       //allow row order to be changed
  // autoColumns:true,
  initialSort:[             //set the initial sort order of the data
      {column:"name", dir:"asc"},
  ],
  columns: coldef,
});
</script>
