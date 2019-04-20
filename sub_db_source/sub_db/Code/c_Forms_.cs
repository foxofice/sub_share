using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace sub_db
{
	internal class c_Forms_
	{
		internal static ToolTip	m_s_Tooltip	= new ToolTip();

		/*==============================================================
		 * 执行委托
		 *==============================================================*/
		internal static void	Invoke(Action func)
		{
			if(c_Mainform.m_s_mainform.InvokeRequired)
				c_Mainform.m_s_mainform.Invoke(func);
			else
				func();
		}
		//--------------------------------------------------
		internal static TResult	Invoke<TResult>(Func<TResult> func)
		{
			if(c_Mainform.m_s_mainform.InvokeRequired)
				return (TResult)c_Mainform.m_s_mainform.Invoke(func);
			else
				return func();
		}

		/*==============================================================
		 * 激活窗口
		 *==============================================================*/
		internal static void	active_form(Form frm)
		{
			frm.Show();
			frm.BringToFront();
			frm.Activate();
			frm.WindowState = FormWindowState.Normal;
		}
	};
}	// namespace sub_db
