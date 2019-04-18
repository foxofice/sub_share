using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.IO;

namespace sub_db
{
	internal class c_Settings_
	{
		static string				m_s_OPTION__SUBS_PATH		= "subs_path";		// 字幕列表路径
		static string				m_s_OPTION__UPDATA_DB_TIME	= "updata_db_time";	// 最后更新数据库的时间

		static internal string		m_s_subs_path				= "../subs_list";	// 字幕列表路径
		static internal DateTime	m_s_updata_db_time			= DateTime.MinValue;// 最后更新数据库的时间

		/*==============================================================
		 * 读取配置文件
		 *==============================================================*/
		static internal void	read_config()
		{
			if(!File.Exists(c_Path_.m_k_CONFIG_FILENAME))
				return;

			string[] lines = File.ReadAllLines(c_Path_.m_k_CONFIG_FILENAME, Encoding.UTF8);

			foreach(string line in lines)
			{
				if(line.Length < 2)
					continue;

				if(line[0] == '/' && line[1] == '/')
					continue;

				int idx = line.IndexOf(":");
				if(idx < 0)
					continue;

				string w1 = line.Substring(0, idx).Trim().ToLower();
				string w2 = line.Substring(idx + 1).Trim();

				if(w1 == m_s_OPTION__SUBS_PATH.ToLower())
				{
					m_s_subs_path = w2;
					continue;
				}

				if(w1 == m_s_OPTION__UPDATA_DB_TIME.ToLower())
				{
					DateTime.TryParse(w2, out m_s_updata_db_time);
					continue;
				}
			}	// for
		}

		/*==============================================================
		 * 写入配置文件
		 *==============================================================*/
		static internal void	write_config()
		{
			List<string> lines = new List<string>();

			lines.Add("// 字幕路径");
			lines.Add(string.Format("subs_path: {0:s}", m_s_subs_path));
			lines.Add("");

			lines.Add("// 最后更新数据库的时间");
			lines.Add(string.Format("updata_db_time: {0:s}", m_s_updata_db_time.ToString()));
			lines.Add("");

			File.WriteAllLines(c_Path_.m_k_CONFIG_FILENAME, lines, Encoding.UTF8);
		}
	};
}	// namespace sub_db
