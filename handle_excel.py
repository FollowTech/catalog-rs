from typing import List, Optional, Tuple

from openpyxl import Workbook, load_workbook


def get_model_name() -> Optional[str]:
    return 'Latitude Pro 13'


# 两个字符串的最长公共子串
def longest_common_substring(s1: str, s2: str) -> str:
    if not isinstance(s1, str) or not isinstance(s2, str):
        raise ValueError(f'Both inputs must be strings, but got {type(s1)} and {type(s2)}')
    s1 = s1.lower()
    s2 = s2.lower()
    m = len(s1)
    n = len(s2)
    # 使用二维数组来存储最长连续公共子串的长度
    dp = [[0] * (n + 1) for _ in range(m + 1)]
    max_len = 0
    end = 0
    for i in range(1, m + 1):
        for j in range(1, n + 1):
            if s1[i - 1] == s2[j - 1]:
                dp[i][j] = dp[i - 1][j - 1] + 1
                if dp[i][j] > max_len:
                    max_len = dp[i][j]
                    end = i  # 更新 end 位置
            else:
                dp[i][j] = 0

    if max_len > 0:
        start = end - max_len
        result = s1[start:end]
    else:
        result = ''

    return result


def get_selected_index(max_len: int) -> int:
    while True:
        _input = input('Press 1 or 2 or 3... to select your project: ')
        try:
            selected_index = int(_input)
        except ValueError:
            print('Please input a number index')
            continue
        if selected_index > max_len:
            print('Please input a valid index')
            continue
        return selected_index


def find_project_name(wb: Workbook, sheet_names: list[str], model_name: str) -> Workbook:
    # device_with_sheet: Dict[str, List[str]] = dict()
    for sheet_name in sheet_names:
        all_project_name: List[Tuple[int, str]] = list()
        if sheet_name.startswith('ModelName'):
            continue
        sheet = wb[f'{sheet_name}']
        index = 1
        _cur = 1
        for cell in sheet[1]:
            if cell.value is None:
                continue
            cell_value = str(cell.value)
            # print(value)
            # print(longest_common_substring(model_name, cell_value.lower()))
            # print(cell_value, '____', model_name)
            if len(longest_common_substring(model_name, cell_value)) > 4:
                all_project_name.append((_cur, f'{index}: {cell_value}'))
                index += 1
            _cur += 1
        if not all_project_name:
            sheet.insert_cols(2)
            sheet.cell(row=1, column=2).value = model_name
            sheet.cell(row=sheet.max_row + 1, column=2).value = 'V'
            continue
        print(sheet_name, '-', [f'{name}' for _, name in all_project_name])
        selected_index = get_selected_index(len(all_project_name))
        selected_project = all_project_name[selected_index - 1]
        sheet_index, sel = selected_project
        # print(selected_project)
        sheet.cell(row=1, column=sheet_index).value = model_name
        # device_with_sheet.setdefault(sheet_name, list()).extend(all_project_name)
    # print(device_with_sheet)
    return wb


wb = load_workbook(filename='example.xlsx')
sheet_names = wb.sheetnames
sheet_modelname = wb['ModelName']
sheet_modelname['A2'] = 'Quake'
sheet_modelname['B2'] = '' if get_model_name() is None else get_model_name()  # type: ignore
Modified_wb = find_project_name(wb, sheet_names, str(sheet_modelname['A2'].value))
wb.save(filename='example.xlsx')
# print(sheet_modelname['B2'].value)


# import unittest


# class TestLongestCommonSubstring(unittest.TestCase):
#     def test_normal_cases(self):
#         # 正常情况
#         self.assertEqual(longest_common_substring('ABCD', 'ACDF'), 'CD')
#         self.assertEqual(longest_common_substring('ABCDEF', 'ZBCDF'), 'BCD')
#         self.assertEqual(longest_common_substring('abcdef', 'abcxyz'), 'abc')
#         self.assertEqual(longest_common_substring('123456', '123789'), '123')

#     def test_empty_strings(self):
#         # 空字符串
#         self.assertEqual(longest_common_substring('', ''), '')
#         self.assertEqual(longest_common_substring('ABC', ''), '')
#         self.assertEqual(longest_common_substring('', 'DEF'), '')

#     def test_no_common_substring(self):
#         # 没有公共子串
#         self.assertEqual(longest_common_substring('ABC', 'XYZ'), '')
#         self.assertEqual(longest_common_substring('123', '456'), '')

#     def test_single_character_strings(self):
#         # 单字符字符串
#         self.assertEqual(longest_common_substring('A', 'A'), 'A')
#         self.assertEqual(longest_common_substring('A', 'B'), '')
#         self.assertEqual(longest_common_substring('A', 'AA'), 'A')

#     def test_long_strings(self):
#         # 较长的字符串
#         s1 = 'This is a very long string that should be tested thoroughly'
#         s2 = 'Another very long string that should also be tested'
#         self.assertEqual(longest_common_substring(s1, s2), ' very long string that should ')

#     def test_special_characters(self):
#         # 特殊字符
#         self.assertEqual(longest_common_substring('a!b@c#', 'a!x@c#'), '@c#')
#         self.assertEqual(longest_common_substring('123!@#', '123$%^'), '123')

#     def test_type_errors(self):
#         # 类型错误
#         with self.assertRaises(ValueError):
#             longest_common_substring(123, 'ABC')
#         with self.assertRaises(ValueError):
#             longest_common_substring('ABC', 123)
#         with self.assertRaises(ValueError):
#             longest_common_substring(123, 456)


# if __name__ == '__main__':
#     unittest.main()
