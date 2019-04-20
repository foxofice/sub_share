using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace sub_db
{
	internal class c_SQL
	{
		/*==============================================================
		 * 修正字符串（转义字符）
		 *==============================================================*/
		internal static string	fix_string(string str)
		{
			return str.Replace("'", "''");
		}
	};
}	// namespace sub_db
