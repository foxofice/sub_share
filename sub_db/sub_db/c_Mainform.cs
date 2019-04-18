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

		internal static c_Mainform	m_s_mainform				= null;
		internal c_UpdateDatabase	m_UpdateDatabase			= new c_UpdateDatabase();
		internal c_Search			m_Search					= new c_Search();
		internal c_Setting			m_Setting					= new c_Setting();
		internal c_About			m_About						= new c_About();

		// DataGridView 的事件是否有效
		internal bool				m_DataGridView_event_enable	= true;

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
			c_Config_.read_config();

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
		 * 调整窗口结束
		 *==============================================================*/
		private void c_Mainform_ResizeEnd(object sender, EventArgs e)
		{
			c_Config_.m_s_window_width	= this.Width;
			c_Config_.m_s_window_height	= this.Height;
			c_Config_.m_s_window_x		= this.DesktopLocation.X;
			c_Config_.m_s_window_y		= this.DesktopLocation.Y;

			c_Config_.write_config();
		}

		/*==============================================================
		 * 查找（回车）
		 *==============================================================*/
		private void TextBox_Filter_KeyPress(object sender, KeyPressEventArgs e)
		{
			if(e.KeyChar == '\r')
				do_search();
		}

		/*==============================================================
		 * 查找（按钮）
		 *==============================================================*/
		private void PictureBox_Search_Click(object sender, EventArgs e)
		{
			do_search();
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

		/*==============================================================
		 * 调整列宽
		 *==============================================================*/
		private void DataGridView_Main_ColumnWidthChanged(object sender, DataGridViewColumnEventArgs e)
		{
			if(!m_DataGridView_event_enable)
				return;

			for(int i=0; i<(int)c_Data_.e_ColumnName.MAX; ++i)
				c_Config_.m_s_column_width[i] = dataGridView_Main.Columns[((c_Data_.e_ColumnName)i).ToString()].Width;

			c_Config_.write_config();
		}

		/*==============================================================
		 * 调整列顺序
		 *==============================================================*/
		private void DataGridView_Main_ColumnDisplayIndexChanged(object sender, DataGridViewColumnEventArgs e)
		{
			if(!m_DataGridView_event_enable)
				return;

			for(int i=0; i<(int)c_Data_.e_ColumnName.MAX; ++i)
				c_Config_.m_s_column_idx[i] = dataGridView_Main.Columns[((c_Data_.e_ColumnName)i).ToString()].DisplayIndex;

			c_Config_.write_config();
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
		 * 高级查找
		 *==============================================================*/
		private void ToolStripButton_Search_Click(object sender, EventArgs e)
		{
			c_Forms_.active_form(m_Search);
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

		/*==============================================================
		 * 更新 dataGridView_Main 的各列样式
		 *==============================================================*/
		internal void	update_columns_style()
		{
			m_DataGridView_event_enable = false;

			for(int i=0; i<c_Config_.m_s_column_width.Length; ++i)
			{
				if(c_Config_.m_s_column_width[i] > 0)
					dataGridView_Main.Columns[((c_Data_.e_ColumnName)i).ToString()].Width = c_Config_.m_s_column_width[i];
			}

			SortedDictionary<int, string> sort_list = new SortedDictionary<int, string>();

			for(int i=0; i<c_Config_.m_s_column_idx.Length; ++i)
			{
				if(c_Config_.m_s_column_idx[i] >= 0)
					sort_list.Add(c_Config_.m_s_column_idx[i], ((c_Data_.e_ColumnName)i).ToString());
			}

			foreach(var kvp in sort_list)
				dataGridView_Main.Columns[kvp.Value].DisplayIndex = kvp.Key;

			m_DataGridView_event_enable = true;
		}

		/*==============================================================
		 * 根据条件查找数据
		 *==============================================================*/
		internal void	do_search()
		{
			c_Data_.m_s_lock.EnterReadLock();

			if(textBox_Filter.TextLength == 0)
			{
				m_DataGridView_event_enable = false;
				dataGridView_Main.DataSource = c_Data_.m_s_dt;
				update_columns_style();
				m_DataGridView_event_enable = true;
			}
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

					m_DataGridView_event_enable = false;
					dataGridView_Main.DataSource = c_Data_.m_s_dt_search;
					update_columns_style();
					m_DataGridView_event_enable = true;
				}
				catch(Exception ex)
				{
					MessageBox.Show(ex.Message);
				}
			}

			c_Data_.m_s_lock.ExitReadLock();
		}
	};
}	// namespace sub_db
