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

		internal static c_Mainform	m_s_mainform		= null;
		internal c_About			m_About				= new c_About();
		internal c_Setting			m_Setting			= new c_Setting();
		internal c_UpdateDatabase	m_UpdateDatabase	= new c_UpdateDatabase();

		#region Winform 事件
		/*==============================================================
		 * 窗口加载时
		 *==============================================================*/
		private void c_Mainform_Load(object sender, EventArgs e)
		{
			m_s_mainform	= this;

			this.Icon		= c_Image_.img2icon(Resource1.Logo);
			this.Text		= $"{c_Common_.m_k_PROGRAM_NAME} {c_Common_.m_k_VERSION}";

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
			if(MessageBox.Show(	c_Languages_.txt(21),	// 是否要退出程序？
								$"{c_Common_.m_k_PROGRAM_NAME} {c_Common_.m_k_VERSION}",
								MessageBoxButtons.YesNo,
								MessageBoxIcon.Question,
								MessageBoxDefaultButton.Button2 ) == DialogResult.No)
			{
				e.Cancel = true;
				return;
			}
		}

		/*==============================================================
		 * 查找（回车）
		 *==============================================================*/
		private void TextBox_Filter_KeyPress(object sender, KeyPressEventArgs e)
		{
			if(e.KeyChar == '\r')
				PictureBox_Search_Click(null, null);
		}

		/*==============================================================
		 * 查找（按钮）
		 *==============================================================*/
		private void PictureBox_Search_Click(object sender, EventArgs e)
		{
			c_Data_.m_s_lock.EnterReadLock();

			if(textBox_Filter.TextLength == 0)
				dataGridView_Main.DataSource = c_Data_.m_s_dt;
			else
			{
				try
				{
					DataRow[] dr_list = c_Data_.m_s_dt.Select(textBox_Filter.Text);

					c_Data_.m_s_dt_search.Rows.Clear();
					foreach(DataRow dr in dr_list)
					{
						DataRow dr_tmp = c_Data_.m_s_dt_search.NewRow();
						for(int i=0; i<c_Data_.m_s_dt_search.Columns.Count; ++i)
							dr_tmp[i] = dr[i];

						c_Data_.m_s_dt_search.Rows.Add(dr_tmp);
					}

					dataGridView_Main.DataSource = c_Data_.m_s_dt_search;
				}
				catch(Exception ex)
				{
					MessageBox.Show(ex.Message);
				}
			}

			c_Data_.m_s_lock.ExitReadLock();
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

		#region 顶部菜单
		/*==============================================================
		 * 更新数据库
		 *==============================================================*/
		private void ToolStripButton_UpdateDB_Click(object sender, EventArgs e)
		{
			c_Forms_.active_form(m_UpdateDatabase);
		}

		/*==============================================================
		 * 设置
		 *==============================================================*/
		private void ToolStripButton_Settings_Click(object sender, EventArgs e)
		{
			c_Forms_.active_form(m_Setting);
		}

		/*==============================================================
		 * 关于
		 *==============================================================*/
		private void ToolStripButton_About_Click(object sender, EventArgs e)
		{
			c_Forms_.active_form(m_About);
		}
		#endregion
	};
}	// namespace sub_db
