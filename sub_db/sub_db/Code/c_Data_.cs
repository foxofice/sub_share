using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Threading;
using System.Data;

namespace sub_db
{
	internal class c_Data_
	{
		// 分类（animation/movie/...）
		List<string>	m_type_list		= new List<string>();

		// 字幕匹配（DB/DVD/TV/test/...）
		List<string>	m_match_list	= new List<string>();

		// 字幕信息
		internal class c_SubInfo
		{
			// 番剧名称
			internal string		m_name_chs	= "";					// (简体)
			internal string		m_name_cht	= "";					// (繁体)
			internal string		m_name_jp	= "";					// (日文)
			internal string		m_name_en	= "";					// (英文)
			internal string		m_name_rome	= "";					// (罗马字)

			internal DateTime	m_time		= DateTime.MinValue;	// 最早放送日期
			//internal byte		m_type_idx	= 0;					// 分类（m_type_list 的索引）
			//internal byte		m_match_idx	= 0;					// 字幕匹配（m_match_list 的索引）
			internal string		m_type		= "";					// 分类（animation/movie/...）
			internal string		m_match		= "";					// 字幕匹配（DB/DVD/TV/test/...）
			internal string		m_sub_name	= "";					// 字幕名称
			internal string		m_extension	= "";					// 字幕后缀名（多种格式时用;分割，例如：.ass;.srt）
			internal string		m_providers	= "";					// 字幕组/提供者
			internal string		m_desc		= "";					// 字幕说明
		};

		// 字幕列表（按文件顺序读取）
		internal static List<c_SubInfo>			m_s_all_subs	= new List<c_SubInfo>();
		internal static ReaderWriterLockSlim	m_s_lock		= new ReaderWriterLockSlim();

		internal static DataTable				m_s_dt			= new DataTable("subs");
		internal static DataTable				m_s_dt_search	= new DataTable("subs");

		/*==============================================================
		 * 更新 DataTable
		 *==============================================================*/
		internal static void	update_DataTable()
		{
			if(m_s_dt.Columns.Count == 0)
			{
				string[] columns_desc =
				{
					"name_chs",		"System.String",
					"name_cht",		"System.String",
					"name_jp",		"System.String",
					"name_en",		"System.String",
					"name_rome",	"System.String",

					"time",			"System.DateTime",
					"type",			"System.String",
					"match",		"System.String",
					"sub_name",		"System.String",
					"extension",	"System.String",
					"providers",	"System.String",
					"desc",			"System.String",
				};

				for(int i=0; i<columns_desc.Length; i+=2)
				{
					m_s_dt.Columns.Add(columns_desc[i], Type.GetType(columns_desc[i + 1]));
					m_s_dt_search.Columns.Add(columns_desc[i], Type.GetType(columns_desc[i + 1]));
				}

				//m_s_dt.Columns["extension"].SetOrdinal(1);
				//m_s_dt.Columns["type"].SetOrdinal(0);
			}

			m_s_lock.EnterWriteLock();

			m_s_dt.Rows.Clear();

			foreach(c_SubInfo sub_info in m_s_all_subs)
			{
				DataRow dr = m_s_dt.NewRow();
				dr["name_chs"]	= sub_info.m_name_chs;
				dr["name_cht"]	= sub_info.m_name_cht;
				dr["name_jp"]	= sub_info.m_name_jp;
				dr["name_en"]	= sub_info.m_name_en;
				dr["name_rome"]	= sub_info.m_name_rome;

				dr["time"]		= sub_info.m_time;
				dr["type"]		= sub_info.m_type;
				dr["match"]		= sub_info.m_match;
				dr["sub_name"]	= sub_info.m_sub_name;
				dr["extension"]	= sub_info.m_extension;
				dr["providers"]	= sub_info.m_providers;
				dr["desc"]		= sub_info.m_desc;

				m_s_dt.Rows.Add(dr);
			}	// for

			m_s_lock.ExitWriteLock();
		}
	};
}	// namespace sub_db
