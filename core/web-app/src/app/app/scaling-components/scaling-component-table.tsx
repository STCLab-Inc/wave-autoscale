import React, { useEffect, useState } from 'react';

import dayjs from 'dayjs';

import { renderKeyValuePairsWithJson } from '../common/keyvalue-renderer';

interface TableProps {
  data: any[];
  setData: (data: any[]) => void;
  /*  */
  selectAll?: boolean;
  handleSelectAll?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  /*  */
  sizePerPageOptions?: number[];
  sizePerPage: number;
  handleSizePerPage: (event: React.ChangeEvent<HTMLSelectElement>) => void;
  /*  */
  currentPage: number;
  totalPage: number;
  handleCurrentPage: (newPage: number) => void;
  /*  */
  onClickDetails: (item: any) => void;
}

export const TableComponent: React.FC<TableProps> = ({
  data: originalData,
  setData,
  selectAll,
  handleSelectAll,
  sizePerPageOptions = [10, 20, 50, 100, 200, 500],
  sizePerPage,
  handleSizePerPage,
  currentPage,
  totalPage,
  handleCurrentPage,
  onClickDetails,
}) => {
  const [dataPerPage, setDataPerPage] = useState<any[]>([]);
  const [tempCurrentPage, setTempCurrentPage] = useState(currentPage);

  const [cloneData, setCloneData] = useState(
    JSON.parse(JSON.stringify(originalData))
  );

  useEffect(() => {
    setCloneData(JSON.parse(JSON.stringify(originalData)));
  }, [originalData]);

  useEffect(() => {
    const startIndex = (currentPage - 1) * sizePerPage;
    const endIndex = startIndex + sizePerPage;
    setDataPerPage(cloneData.slice(startIndex, endIndex));
  }, [cloneData, currentPage, sizePerPage, totalPage]);

  useEffect(() => {
    setTempCurrentPage(currentPage);
  }, [currentPage]);

  const handleCheckboxChange = (
    event: React.ChangeEvent<HTMLInputElement>,
    dataItem: { id: any }
  ) => {
    const checked = event.target.checked;
    const newData = cloneData;
    const updatedData = newData.map((item: { id: any }) =>
      item.id === dataItem.id ? { ...item, isChecked: checked } : item
    );
    setCloneData(updatedData);
  };

  /*  */

  interface TempItem {
    label: string;
    type: string;
    weight: string;
    content: string;
    format?: string;
    status?: any;
    function?: any;
  }

  const tempData: TempItem[] = [
    {
      label: 'checkbox',
      type: 'checkbox',
      content: 'isChecked',
      weight: '1',
      status: selectAll,
      function: handleSelectAll,
    },
    {
      label: 'Scaling Component ID',
      type: 'span',
      content: 'id',
      format: 'plain',
      weight: '8',
    },
    {
      label: 'Scaling Component Kind',
      type: 'span',
      content: 'component_kind',
      format: 'plain',
      weight: '8',
    },
    {
      label: 'Metadata',
      type: 'span',
      content: 'metadata',
      format: 'json',
      weight: '10',
    },
    {
      label: 'Actions',
      type: 'button',
      content: 'dataItem',
      format: 'click',
      weight: '1',
      function: onClickDetails,
    },
  ];

  return (
    <div className="flex w-full flex-col">
      <div className="flex items-center justify-end px-8 py-4">
        <div className="mx-4 flex h-8 items-center">
          <div className="mx-2 flex items-center">
            <label className="select-group-sm">
              <select
                value={sizePerPage}
                onChange={handleSizePerPage}
                className="focus:outline-noneselect select-sm max-w-[130px] cursor-pointer rounded-md border border-gray-200 px-2"
              >
                {sizePerPageOptions.map((option, index) => (
                  <option key={index} value={option}>
                    {option}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <div className="min-h-8 mx-2 flex min-w-[100px] items-center justify-center rounded-md border border-gray-200 px-2">
            <input
              className="w-[60px] min-w-[60px] text-center text-sm"
              value={tempCurrentPage}
              onChange={(event) =>
                setTempCurrentPage(Number(event?.target.value))
              }
            />
            <span className="px-2 text-center text-sm">/</span>
            <span className="min-w-[60px] px-4 text-center text-sm">
              {totalPage}
            </span>
          </div>
          <button
            className={
              tempCurrentPage > totalPage || tempCurrentPage < 1
                ? 'ml-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                : 'ml-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50'
            }
            onClick={() => handleCurrentPage(tempCurrentPage)}
            disabled={tempCurrentPage > totalPage || tempCurrentPage < 1}
          >
            MOVE
          </button>
        </div>

        <div className="ml-2 flex h-8 items-center">
          <button
            className={
              currentPage && totalPage && currentPage === 1
                ? 'mr-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                : 'mr-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-red-400 bg-red-400 pl-5 pr-5 text-sm text-gray-50'
            }
            onClick={() => {
              handleCurrentPage(currentPage - 1);
            }}
            disabled={currentPage === 1}
          >
            PREVIOUS
          </button>
          <button
            className={
              currentPage && totalPage && currentPage === totalPage
                ? 'ml-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                : 'ml-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50'
            }
            onClick={() => {
              handleCurrentPage(currentPage + 1);
            }}
            disabled={currentPage === totalPage}
          >
            NEXT
          </button>
        </div>
      </div>

      <table className="flex w-full flex-col">
        <thead className="text-md flex h-12 w-full items-center justify-between border-b border-t bg-gray-75 py-0 font-bold text-gray-800">
          <tr className="flex h-full w-full px-8">
            {tempData.map((item, index) => {
              if (item.type === 'checkbox') {
                return (
                  <th
                    key={index}
                    className={`flex h-full pr-4 flex-${item.weight} items-center`}
                  >
                    <label className="flex h-full items-center">
                      <input
                        type="checkbox"
                        className="checkbox"
                        checked={item.status}
                        onChange={item.function}
                      />
                    </label>
                  </th>
                );
              } else if (item.type === 'span' || item.type === 'button') {
                return (
                  <th
                    key={index}
                    className={`flex h-full w-full px-4 flex-${item.weight} items-center`}
                  >
                    <span className="flex items-center break-words">
                      {item.label}
                    </span>
                  </th>
                );
              }
            })}
          </tr>
        </thead>
        <tbody className="text-md min-h-12 flex w-full flex-col items-center justify-between py-0 text-gray-800">
          {dataPerPage.map((dataItem: any) => (
            <tr
              /* Warning: Each child in a list should have a unique "key" prop. */
              key={dataItem.id}
              className="flex w-full border-b px-8 py-4"
            >
              {tempData.map((item, index) => {
                if (item.type === 'checkbox') {
                  return (
                    <td
                      key={index}
                      className={`flex h-full pr-4 flex-${item.weight} items-start`}
                    >
                      <label className="flex items-center">
                        <input
                          type="checkbox"
                          className="checkbox"
                          checked={dataItem[item.content]}
                          onChange={(event) =>
                            handleCheckboxChange(event, dataItem)
                          }
                        />
                      </label>
                    </td>
                  );
                } else if (item.type === 'span') {
                  return (
                    <td
                      key={index}
                      className={`flex h-full w-full px-4 flex-${item.weight} items-start`}
                    >
                      {item.format === 'json' ? (
                        <span className="flex flex-col items-center break-all text-start">
                          {dataItem[item.content] &&
                            renderKeyValuePairsWithJson(
                              dataItem[item.content],
                              true
                            )}
                        </span>
                      ) : item.format === 'unix' ? (
                        <span className="flex flex-col items-center break-all text-start">
                          {dayjs
                            .unix(Number(dataItem[item.content]) / 1000)
                            .format('YYYY/MM/DD HH:mm:ss')}
                        </span>
                      ) : item.format === 'plain' ? (
                        <span className="flex flex-col items-center break-all text-start">
                          {dataItem[item.content]}
                        </span>
                      ) : (
                        <span className="flex flex-col items-center break-all text-start">
                          {dataItem[item.content]}
                        </span>
                      )}
                    </td>
                  );
                } else if (item.type === 'button') {
                  return (
                    <td
                      key={index}
                      className={`flex h-full w-full px-4 flex-${item.weight} items-start`}
                    >
                      {item.format === 'alternate' ? (
                        <div className="flex items-center">
                          {dataItem[item.content] ? (
                            <button
                              className="badge-error badge bg-[#E0242E] px-2 py-3 text-white"
                              onClick={() => item.function(dataItem)}
                            >
                              Failed
                            </button>
                          ) : (
                            <button
                              className="badge-success badge bg-[#074EAB] px-2 py-3 text-white"
                              onClick={() => item.function(dataItem)}
                            >
                              Success
                            </button>
                          )}
                        </div>
                      ) : item.format === 'click' ? (
                        <div className="flex items-center">
                          <button
                            className="badge-info badge bg-[#99E3D0] px-2 py-3 text-white"
                            onClick={() => item.function(dataItem)}
                          >
                            Details
                          </button>
                        </div>
                      ) : null}
                    </td>
                  );
                }
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};
