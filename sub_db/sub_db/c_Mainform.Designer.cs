namespace sub_db
{
	partial class c_Mainform
	{
		/// <summary>
		/// 必需的设计器变量。
		/// </summary>
		private System.ComponentModel.IContainer components = null;

		/// <summary>
		/// 清理所有正在使用的资源。
		/// </summary>
		/// <param name="disposing">如果应释放托管资源，为 true；否则为 false。</param>
		protected override void Dispose(bool disposing)
		{
			if(disposing && (components != null))
			{
				components.Dispose();
			}
			base.Dispose(disposing);
		}

		#region Windows 窗体设计器生成的代码

		/// <summary>
		/// 设计器支持所需的方法 - 不要修改
		/// 使用代码编辑器修改此方法的内容。
		/// </summary>
		private void InitializeComponent()
		{
			this.dataGridView_Main = new System.Windows.Forms.DataGridView();
			this.textBox_Filter = new System.Windows.Forms.TextBox();
			this.label_Filter = new System.Windows.Forms.Label();
			this.linkLabel_FilterHelp = new System.Windows.Forms.LinkLabel();
			this.toolStrip_Main = new System.Windows.Forms.ToolStrip();
			this.toolStripSeparator1 = new System.Windows.Forms.ToolStripSeparator();
			this.toolStripButton_UpdateDB = new System.Windows.Forms.ToolStripButton();
			this.toolStripSplitButton_Languages = new System.Windows.Forms.ToolStripSplitButton();
			this.toolStripButton_Settings = new System.Windows.Forms.ToolStripButton();
			this.toolStripButton_About = new System.Windows.Forms.ToolStripButton();
			this.pictureBox_Search = new System.Windows.Forms.PictureBox();
			((System.ComponentModel.ISupportInitialize)(this.dataGridView_Main)).BeginInit();
			this.toolStrip_Main.SuspendLayout();
			((System.ComponentModel.ISupportInitialize)(this.pictureBox_Search)).BeginInit();
			this.SuspendLayout();
			// 
			// dataGridView_Main
			// 
			this.dataGridView_Main.AllowUserToAddRows = false;
			this.dataGridView_Main.AllowUserToDeleteRows = false;
			this.dataGridView_Main.AllowUserToOrderColumns = true;
			this.dataGridView_Main.Anchor = ((System.Windows.Forms.AnchorStyles)((((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Bottom) 
            | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
			this.dataGridView_Main.ColumnHeadersHeightSizeMode = System.Windows.Forms.DataGridViewColumnHeadersHeightSizeMode.AutoSize;
			this.dataGridView_Main.Location = new System.Drawing.Point(0, 55);
			this.dataGridView_Main.Name = "dataGridView_Main";
			this.dataGridView_Main.ReadOnly = true;
			this.dataGridView_Main.RowTemplate.Height = 23;
			this.dataGridView_Main.Size = new System.Drawing.Size(784, 506);
			this.dataGridView_Main.TabIndex = 2;
			this.dataGridView_Main.ColumnDisplayIndexChanged += new System.Windows.Forms.DataGridViewColumnEventHandler(this.DataGridView_Main_ColumnDisplayIndexChanged);
			this.dataGridView_Main.ColumnWidthChanged += new System.Windows.Forms.DataGridViewColumnEventHandler(this.DataGridView_Main_ColumnWidthChanged);
			// 
			// textBox_Filter
			// 
			this.textBox_Filter.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
			this.textBox_Filter.Location = new System.Drawing.Point(83, 28);
			this.textBox_Filter.Name = "textBox_Filter";
			this.textBox_Filter.Size = new System.Drawing.Size(650, 21);
			this.textBox_Filter.TabIndex = 1;
			this.textBox_Filter.KeyPress += new System.Windows.Forms.KeyPressEventHandler(this.TextBox_Filter_KeyPress);
			// 
			// label_Filter
			// 
			this.label_Filter.AutoSize = true;
			this.label_Filter.Location = new System.Drawing.Point(12, 31);
			this.label_Filter.Name = "label_Filter";
			this.label_Filter.Size = new System.Drawing.Size(65, 12);
			this.label_Filter.TabIndex = 0;
			this.label_Filter.Text = "查询语句：";
			// 
			// linkLabel_FilterHelp
			// 
			this.linkLabel_FilterHelp.Anchor = ((System.Windows.Forms.AnchorStyles)((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Right)));
			this.linkLabel_FilterHelp.AutoSize = true;
			this.linkLabel_FilterHelp.Location = new System.Drawing.Point(761, 31);
			this.linkLabel_FilterHelp.Name = "linkLabel_FilterHelp";
			this.linkLabel_FilterHelp.Size = new System.Drawing.Size(17, 12);
			this.linkLabel_FilterHelp.TabIndex = 4;
			this.linkLabel_FilterHelp.TabStop = true;
			this.linkLabel_FilterHelp.Text = "？";
			this.linkLabel_FilterHelp.LinkClicked += new System.Windows.Forms.LinkLabelLinkClickedEventHandler(this.LinkLabel_FilterHelp_LinkClicked);
			// 
			// toolStrip_Main
			// 
			this.toolStrip_Main.Items.AddRange(new System.Windows.Forms.ToolStripItem[] {
            this.toolStripButton_UpdateDB,
            this.toolStripSplitButton_Languages,
            this.toolStripButton_Settings,
            this.toolStripSeparator1,
            this.toolStripButton_About});
			this.toolStrip_Main.Location = new System.Drawing.Point(0, 0);
			this.toolStrip_Main.Name = "toolStrip_Main";
			this.toolStrip_Main.Size = new System.Drawing.Size(784, 25);
			this.toolStrip_Main.TabIndex = 7;
			this.toolStrip_Main.Text = "toolStrip1";
			// 
			// toolStripSeparator1
			// 
			this.toolStripSeparator1.Name = "toolStripSeparator1";
			this.toolStripSeparator1.Size = new System.Drawing.Size(6, 25);
			// 
			// toolStripButton_UpdateDB
			// 
			this.toolStripButton_UpdateDB.Image = global::sub_db.Resource1.DB_Refresh;
			this.toolStripButton_UpdateDB.ImageTransparentColor = System.Drawing.Color.Magenta;
			this.toolStripButton_UpdateDB.Name = "toolStripButton_UpdateDB";
			this.toolStripButton_UpdateDB.Size = new System.Drawing.Size(88, 22);
			this.toolStripButton_UpdateDB.Text = "更新数据库";
			this.toolStripButton_UpdateDB.Click += new System.EventHandler(this.ToolStripButton_UpdateDB_Click);
			// 
			// toolStripSplitButton_Languages
			// 
			this.toolStripSplitButton_Languages.Image = global::sub_db.Resource1.Languages;
			this.toolStripSplitButton_Languages.ImageTransparentColor = System.Drawing.Color.Magenta;
			this.toolStripSplitButton_Languages.Name = "toolStripSplitButton_Languages";
			this.toolStripSplitButton_Languages.Size = new System.Drawing.Size(135, 22);
			this.toolStripSplitButton_Languages.Text = "语言(Languages)";
			// 
			// toolStripButton_Settings
			// 
			this.toolStripButton_Settings.Image = global::sub_db.Resource1.Settings;
			this.toolStripButton_Settings.ImageTransparentColor = System.Drawing.Color.Magenta;
			this.toolStripButton_Settings.Name = "toolStripButton_Settings";
			this.toolStripButton_Settings.Size = new System.Drawing.Size(52, 22);
			this.toolStripButton_Settings.Text = "设置";
			this.toolStripButton_Settings.Click += new System.EventHandler(this.ToolStripButton_Settings_Click);
			// 
			// toolStripButton_About
			// 
			this.toolStripButton_About.Image = global::sub_db.Resource1.About;
			this.toolStripButton_About.ImageTransparentColor = System.Drawing.Color.Magenta;
			this.toolStripButton_About.Name = "toolStripButton_About";
			this.toolStripButton_About.Size = new System.Drawing.Size(52, 22);
			this.toolStripButton_About.Text = "关于";
			this.toolStripButton_About.Click += new System.EventHandler(this.ToolStripButton_About_Click);
			// 
			// pictureBox_Search
			// 
			this.pictureBox_Search.Anchor = ((System.Windows.Forms.AnchorStyles)((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Right)));
			this.pictureBox_Search.Image = global::sub_db.Resource1.Search;
			this.pictureBox_Search.Location = new System.Drawing.Point(739, 30);
			this.pictureBox_Search.Name = "pictureBox_Search";
			this.pictureBox_Search.Size = new System.Drawing.Size(16, 16);
			this.pictureBox_Search.SizeMode = System.Windows.Forms.PictureBoxSizeMode.AutoSize;
			this.pictureBox_Search.TabIndex = 5;
			this.pictureBox_Search.TabStop = false;
			this.pictureBox_Search.Click += new System.EventHandler(this.PictureBox_Search_Click);
			// 
			// c_Mainform
			// 
			this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 12F);
			this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
			this.ClientSize = new System.Drawing.Size(784, 561);
			this.Controls.Add(this.toolStrip_Main);
			this.Controls.Add(this.pictureBox_Search);
			this.Controls.Add(this.linkLabel_FilterHelp);
			this.Controls.Add(this.label_Filter);
			this.Controls.Add(this.textBox_Filter);
			this.Controls.Add(this.dataGridView_Main);
			this.Name = "c_Mainform";
			this.StartPosition = System.Windows.Forms.FormStartPosition.CenterScreen;
			this.Text = "Subtitles DataBase";
			this.FormClosing += new System.Windows.Forms.FormClosingEventHandler(this.c_Mainform_FormClosing);
			this.Load += new System.EventHandler(this.c_Mainform_Load);
			((System.ComponentModel.ISupportInitialize)(this.dataGridView_Main)).EndInit();
			this.toolStrip_Main.ResumeLayout(false);
			this.toolStrip_Main.PerformLayout();
			((System.ComponentModel.ISupportInitialize)(this.pictureBox_Search)).EndInit();
			this.ResumeLayout(false);
			this.PerformLayout();

		}

		#endregion
		private System.Windows.Forms.TextBox textBox_Filter;
		private System.Windows.Forms.LinkLabel linkLabel_FilterHelp;
		internal System.Windows.Forms.Label label_Filter;
		internal System.Windows.Forms.DataGridView dataGridView_Main;
		private System.Windows.Forms.PictureBox pictureBox_Search;
		private System.Windows.Forms.ToolStrip toolStrip_Main;
		private System.Windows.Forms.ToolStripSeparator toolStripSeparator1;
		internal System.Windows.Forms.ToolStripSplitButton toolStripSplitButton_Languages;
		internal System.Windows.Forms.ToolStripButton toolStripButton_UpdateDB;
		internal System.Windows.Forms.ToolStripButton toolStripButton_Settings;
		internal System.Windows.Forms.ToolStripButton toolStripButton_About;
	}
}

