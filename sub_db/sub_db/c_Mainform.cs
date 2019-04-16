using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace sub_db
{
	public partial class c_Mainform : Form
	{
		public c_Mainform()
		{
			InitializeComponent();
		}

		internal static c_Mainform	m_s_mainform	= null;
		internal c_About			m_About			= new c_About();
		internal c_Setting			m_Setting		= new c_Setting();

		#region Winform 事件
		/*==============================================================
		 * 窗口加载时
		 *==============================================================*/
		private void c_Mainform_Load(object sender, EventArgs e)
		{
			m_s_mainform			= this;

			this.Icon				= c_Image_.img2icon(Resource1.Logo);
			notifyIcon_Main.Icon	= c_Image_.img2icon(Resource1.Logo);

			this.Text				= $"{c_Common_.m_k_PROGRAM_NAME} {c_Common_.m_k_VERSION}";
			notifyIcon_Main.Text	= this.Text;

			// 读取配置文件
			c_Settings_.read_config();

			// 加载多语言模板
			c_Languages_.read_languages_list();
			c_Languages_.change_language_to_default();
		}

		/*==============================================================
		 * 窗口关闭时
		 *==============================================================*/
		private void c_Mainform_FormClosing(object sender, FormClosingEventArgs e)
		{
			e.Cancel = true;
			this.Hide();
		}

		/*==============================================================
		 * 查询语法帮助
		 *==============================================================*/
		private void LinkLabel_FilterHelp_LinkClicked(object sender, LinkLabelLinkClickedEventArgs e)
		{
			try
			{
				System.Diagnostics.Process.Start("https://www.csharp-examples.net/dataview-rowfilter/");
			}
			catch(Exception)
			{
			}
		}
		#endregion

		#region 托盘图标
		/*==============================================================
		 * 双击托盘图标
		 *==============================================================*/
		private void NotifyIcon_Main_DoubleClick(object sender, EventArgs e)
		{
			if(!this.Visible)
				c_Forms_.active_form(this);
			else
				this.Hide();
		}

		/*==============================================================
		 * 打开
		 *==============================================================*/
		private void ToolStripMenuItem_Open_Click(object sender, EventArgs e)
		{
			c_Forms_.active_form(this);
		}

		/*==============================================================
		 * 更新数据库
		 *==============================================================*/
		private void ToolStripMenuItem_UpdateDB_Click(object sender, EventArgs e)
		{
			// TODO:
		}

		/*==============================================================
		 * 设置
		 *==============================================================*/
		private void ToolStripMenuItem_Settings_Click(object sender, EventArgs e)
		{
			c_Forms_.active_form(m_Setting);
		}

		/*==============================================================
		 * 关于
		 *==============================================================*/
		private void ToolStripMenuItem_About_Click(object sender, EventArgs e)
		{
			c_Forms_.active_form(m_About);
		}

		/*==============================================================
		 * 退出程序
		 *==============================================================*/
		private void ToolStripMenuItem_Exit_Click(object sender, EventArgs e)
		{
			if(MessageBox.Show(	c_Languages_.txt(21),	// 是否要退出程序？
								$"{c_Common_.m_k_PROGRAM_NAME} {c_Common_.m_k_VERSION}",
								MessageBoxButtons.YesNo,
								MessageBoxIcon.Question,
								MessageBoxDefaultButton.Button2 ) == DialogResult.No)
				return;

			notifyIcon_Main.Visible = false;

			c_Common_.quit_program();
		}
		#endregion
	};
}	// namespace sub_db
