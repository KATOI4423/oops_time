import html
import re
import sys
from bs4 import BeautifulSoup
from collections import defaultdict


def parse_license_html(file_path):
	"""
	`cargo about` が生成した HTML ファイルを解析し、
	ライセンスごとに crate（依存ライブラリ）情報をまとめた辞書を返す。

	Args:
		file_path (str): 入力HTMLファイルのパス

	Returns:
		dict[str, dict]: ライセンスIDごとの情報を格納した辞書。
			- キー: ライセンスID（例: "Apache-2.0", "MIT"）
			- 値: {
				"name": ライセンス名（例: "Apache License 2.0"）,
				"used_by": Crate情報のリスト,
				"text": ライセンス本文（<pre>内）
			}
	"""
	with open(file_path, "r", encoding="utf-8") as f:
		soup = BeautifulSoup(f, "html.parser")

	licenses = {}

	# ライセンスごとの <li class="license"> を処理
	for li in soup.select("li.license"):
		h3 = li.find("h3")
		if not h3 or not h3.has_attr("id"):
			continue

		lic_id = h3['id']
		lic_name = h3.get_text(strip=True)

		# 使用しているcrateの一覧を収集
		used_by = []
		for li_tag in li.select("ul.license-used-by li"):
			a = li_tag.find("a")
			if a:
				href = a.get("href", "").strip()
				crate_text = a.get_text(strip=True)
				parts = crate_text.split()
				if len(parts) >= 2:
					name = parts[0]
					version = " ".join(parts[1:])
				else:
					name = crate_text
					version = ""

			used_by.append({
				"name": name,
				"version": version,
				"repository": href
			})

		# ライセンス本文の取得（<pre class="license-text">）
		license_text_tag = li.find("pre", class_="license-text")
		if license_text_tag:
			license_text = license_text_tag.string or license_text_tag.get_text(strip=False)
		else:
			license_text = ""

		# ライセンスIDをキーにして辞書に格納
		if lic_id not in licenses:
			licenses[lic_id] = {
				"name": lic_name,
				"used_by": used_by,
				"text": license_text
			}
		else:
			licenses[lic_id]['used_by'].extend(used_by)

	return licenses


def extract_handlebars_block(template: str, block_name: str) -> tuple[str, str, str]:
	"""
	Handlebarsの `#each` ブロック（繰り返し部分）を抽出して返す。

	Args:
		template (str): テンプレート文字列
		block_name (str): ブロック名（例: "overview", "licenses"）

	Returns:
		tuple[str, str, str]: (全体ブロック, 中身だけ, ブロックをプレースホルダに置き換えたテンプレート)
	"""
	open_tag = "{{#each"
	start_tag = f"{open_tag} {block_name}" + "}}"
	end_tag = "{{/each}}"
	start_idx = template.find(start_tag)
	if start_tag == -1:
		raise ValueError(f"Not found {start_tag}")

	index = start_idx + len(start_tag)
	depth = 1

	# ネストされたeachにも対応するため、開始・終了タグのバランスを取る
	while depth > 0:
		next_open = template.find(open_tag, index)
		next_close = template.find(end_tag, index)
		if next_close == -1:
			raise ValueError(f"Not found {end_tag}")

		if next_open != -1 and next_open < next_close:
			depth += 1
			index = next_open + len(open_tag)
		else:
			depth -= 1
			index = next_close + len(end_tag)

	full_block = template[start_idx:index]
	inner_content = template[start_idx + len(start_tag): index - len(end_tag)]

	# インデントが重複しないように、\n以降に置き換えプレースホルダを挿入
	newline_idx = template.rfind("\n", 0, start_idx)
	remainder = template[:newline_idx+1] + "{{" + f"{block_name}_block" + "}}" + template[index:]

	return full_block, inner_content, remainder


def load_template(template_path: str):
	"""
	テンプレートファイルを読み取り、overview/licenses の each ブロックを抜き出す

	Args:
		template_path (str): テンプレートファイルのパス

	Returns:
		tuple[str, str, str]: base_template, overview_template, licenses_template
	"""
	with open(template_path, "r", encoding="utf-8") as f:
		raw = f.read()

	_, overview_tpl, temp = extract_handlebars_block(raw, "overview")
	_, licenses_tpl, base_tpl = extract_handlebars_block(temp, "licenses")

	return base_tpl, overview_tpl, licenses_tpl


def render_crate_line_simple(template_line: str, crate: dict) -> str:
	"""
	Handlebarsの if/else 文を使わずに、crate情報をシンプルにHTMLへ展開する

	Args:
		template_line (str): 1行テンプレート
		crate (dict): {"name": str, "version": str, "repository": str}

	Returns:
		str: 展開済みのHTML文字列
	"""
	# 条件分岐 (repositoryの有無) を無視して、常に repository を使用
	template_line = re.sub(
		r"{{#if crate\.repository}}.*?{{else}}.*?{{/if}}",
		html.escape(crate['repository']),
		template_line,
		flags=re.DOTALL
	)

	template_line = template_line.replace("{{crate.name}}", html.escape(crate['name']))
	template_line = template_line.replace("{{crate.version}}", html.escape(crate['version']))

	return template_line


def remove_blank_lines_preserving_indent(text: str) -> str:
	"""
	空行を削除し、インデントは維持する

	Args:
		text (str): 入力テキスト

	Returns:
		str: 空行を除いた文字列
	"""
	return "\n".join(line for line in text.splitlines() if line.strip() != "")


def apply_template_preserving_lines(template: str, replacements: dict) -> str:
	"""
	各行のインデントを保持したまま {{key}} を置換する

	Args:
		template (str): テンプレート文字列
		replacements (dict): キーと値の置換マップ

	Returns:
		str: 置換後のテンプレート文字列
	"""
	output_lines = []

	for line in template.splitlines():
		for k, v in replacements.items():
			line = line.replace("{{" + f"{k}" + "}}", v)
		output_lines.append(line)

	return "\n".join(output_lines)


def generate_clean_html(licenses_dict: dict, template_path: str):
	"""
	辞書形式のライセンス情報を元に、Handlebarsテンプレートに展開してHTMLを生成

	Args:
		licenses_dict (dict): ライセンスごとのcrate情報を含む辞書
		template_path (str): テンプレートファイルパス

	Returns:
		str: 出力HTML
	"""
	base_tpl, overview_tpl, licenses_tpl = load_template(template_path)

	overview_html = ""
	licenses_html = ""

	# 各ライセンスごとに overview・license セクションを生成
	for lic_id, lic in licenses_dict.items():
		# overviewブロックを展開
		overview_block = apply_template_preserving_lines(overview_tpl, {
			"id": lic_id,
			"name": html.escape(lic['name']),
			"count": str(len(lic['used_by']))
		})
		overview_html += overview_block + "\n"

		# licensesブロックの中にある used_by のテンプレートを探す
		lic_block = licenses_tpl
		used_by_tpl_match = re.search(r"{{#each used_by}}(.*?){{/each}}", lic_block, re.DOTALL)
		if not used_by_tpl_match:
			continue
		used_by_tpl = used_by_tpl_match.group(1)

		# used_by crate を1行ずつ展開
		used_by_html = ""
		for crate in lic['used_by']:
			used_by_html += render_crate_line_simple(used_by_tpl, crate)

		# 展開したcrateリストと、ライセンス情報を反映
		lic_block = re.sub(r"{{#each used_by}}.*?{{/each}}", used_by_html, lic_block, flags=re.DOTALL)
		lic_block = apply_template_preserving_lines(lic_block, {
			"id": lic_id,
			"name": html.escape(lic['name']),
			"text": html.escape(lic['text'])
		})
		licenses_html += lic_block + "\n"

	# ベーステンプレートに overview, licenses を埋め込む
	final_html = base_tpl.replace("{{overview_block}}", remove_blank_lines_preserving_indent(overview_html))
	final_html = final_html.replace("{{licenses_block}}", remove_blank_lines_preserving_indent(licenses_html))

	return final_html


def main():
	"""
	コマンドライン引数を処理し、HTMLを読み込んで整形・出力
	"""
	if len(sys.argv) < 4:
		print("Usage: python script.py <input_Licenses.html> <template.hbs> <output_Licenses.html>")
		sys.exit(1)

	input_file_path = sys.argv[1]
	template_path = sys.argv[2]
	output_file_path = sys.argv[3]

	# HTMLをパースし、ライセンス情報辞書を構築
	licenses = parse_license_html(input_file_path)

	# テンプレートに適用し、最終HTMLを生成
	final_html = generate_clean_html(licenses, template_path)

	# 出力ファイルに書き込む
	with open(output_file_path, "w", encoding="utf-8") as f:
		f.write(final_html)


if __name__ == "__main__":
	main()
