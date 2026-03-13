//! Dictionary management module
//!
//! Handles system dictionary, user dictionary, and frequency management.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::config::DictionaryConfig;

/// Dictionary entry with frequency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictEntry {
    /// The word/phrase
    pub word: String,
    /// Pinyin representation
    pub pinyin: String,
    /// Usage frequency
    pub frequency: u64,
    /// Last used timestamp
    pub last_used: Option<u64>,
}

/// Dictionary manager
pub struct Dictionary {
    /// System dictionary (read-only)
    system_dict: HashMap<String, Vec<DictEntry>>,
    /// User dictionary (read-write)
    user_dict: RwLock<HashMap<String, Vec<DictEntry>>>,
    /// Configuration
    config: DictionaryConfig,
}

impl Dictionary {
    /// Create a new dictionary manager
    pub fn new(config: DictionaryConfig) -> Self {
        let mut dict = Self {
            system_dict: HashMap::new(),
            user_dict: RwLock::new(HashMap::new()),
            config,
        };

        dict.load_system_dictionary();
        dict.load_user_dictionary();

        dict
    }

    /// Load system dictionary from file
    fn load_system_dictionary(&mut self) {
        let path = expand_path(&self.config.system_dictionary);

        // Try multiple paths in order
        let paths_to_try = vec![
            path.clone(),
            // Development path
            PathBuf::from("data/system.dict"),
            // Relative to crate
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/system.dict"),
        ];

        for try_path in paths_to_try {
            if let Ok(entries) = load_dictionary_file(&try_path) {
                self.system_dict = entries;
                log::info!(
                    "Loaded system dictionary from {:?} with {} entries",
                    try_path,
                    self.system_dict.len()
                );
                return;
            }
        }

        // Initialize with default entries if no dictionary found
        log::warn!("No system dictionary found, using built-in defaults");
        self.init_default_dictionary();
    }

    /// Load user dictionary from file
    fn load_user_dictionary(&mut self) {
        let path = expand_path(&self.config.user_dictionary);
        if let Ok(entries) = load_dictionary_file(&path) {
            let mut user_dict = self.user_dict.write();
            *user_dict = entries;
            log::info!("Loaded user dictionary with {} entries", user_dict.len());
        }
    }

    /// Initialize with default common words
    fn init_default_dictionary(&mut self) {
        let default_words = vec![
            // Common single characters
            ("a", vec!["啊", "阿", "呵"]),
            ("ai", vec!["爱", "艾", "哎", "唉"]),
            ("an", vec!["安", "按", "暗", "岸"]),
            ("ba", vec!["八", "把", "爸", "吧", "巴"]),
            ("bai", vec!["白", "百", "败", "拜"]),
            ("ban", vec!["办", "班", "般", "版", "半"]),
            ("bang", vec!["帮", "邦", "榜", "棒"]),
            ("bao", vec!["包", "报", "保", "宝", "抱"]),
            ("bei", vec!["北", "被", "背", "杯", "备"]),
            ("ben", vec!["本", "奔", "笨"]),
            ("bi", vec!["比", "必", "笔", "毕", "币"]),
            ("bian", vec!["变", "便", "边", "编", "遍"]),
            ("biao", vec!["表", "标", "彪"]),
            ("bie", vec!["别", "憋"]),
            ("bing", vec!["并", "病", "兵", "冰"]),
            ("bo", vec!["不", "波", "博", "播", "薄"]),
            ("bu", vec!["不", "布", "步", "部", "补"]),
            ("ca", vec!["擦"]),
            ("cai", vec!["才", "材", "财", "采", "菜"]),
            ("can", vec!["参", "残", "餐", "惨"]),
            ("cang", vec!["藏", "仓", "苍"]),
            ("cao", vec!["草", "操", "曹"]),
            ("ce", vec!["策", "测", "侧", "册"]),
            ("ceng", vec!["层", "曾"]),
            ("cha", vec!["查", "茶", "差", "插", "察"]),
            ("chai", vec!["差", "柴", "拆"]),
            ("chan", vec!["产", "缠", "禅", "颤"]),
            ("chang", vec!["长", "常", "场", "唱", "厂"]),
            ("chao", vec!["超", "朝", "潮", "吵", "抄"]),
            ("che", vec!["车", "扯", "彻"]),
            ("chen", vec!["陈", "晨", "沉", "尘"]),
            ("cheng", vec!["成", "城", "程", "称", "承"]),
            ("chi", vec!["吃", "持", "迟", "池", "尺"]),
            ("chong", vec!["冲", "虫", "重", "充"]),
            ("chou", vec!["抽", "愁", "丑", "臭"]),
            ("chu", vec!["出", "处", "初", "除", "楚"]),
            ("chuan", vec!["传", "穿", "船", "川"]),
            ("chuang", vec!["创", "床", "窗", "闯"]),
            ("chui", vec!["吹", "锤", "垂"]),
            ("chun", vec!["春", "纯", "唇"]),
            ("ci", vec!["次", "此", "词", "辞"]),
            ("cong", vec!["从", "聪", "丛", "匆"]),
            ("cu", vec!["粗", "促", "醋"]),
            ("cuan", vec!["窜", "攒"]),
            ("cui", vec!["催", "脆", "翠"]),
            ("cun", vec!["村", "存", "寸"]),
            ("cuo", vec!["错", "措", "搓"]),
            ("da", vec!["大", "打", "达", "答"]),
            ("dai", vec!["大", "代", "带", "待", "袋"]),
            ("dan", vec!["但", "单", "淡", "蛋", "担"]),
            ("dang", vec!["当", "党", "档", "挡"]),
            ("dao", vec!["到", "道", "刀", "倒", "导"]),
            ("de", vec!["的", "得", "德", "地"]),
            ("deng", vec!["等", "灯", "登"]),
            ("di", vec!["地", "的", "第", "低", "底"]),
            ("dian", vec!["点", "电", "店", "典", "垫"]),
            ("diao", vec!["掉", "调", "吊", "雕"]),
            ("die", vec!["跌", "爹", "碟", "蝶"]),
            ("ding", vec!["定", "顶", "订", "丁"]),
            ("dong", vec!["动", "东", "冬", "懂", "冻"]),
            ("dou", vec!["都", "斗", "读", "豆"]),
            ("du", vec!["读", "度", "都", "毒", "独"]),
            ("duan", vec!["段", "短", "断", "端"]),
            ("dui", vec!["对", "队", "堆"]),
            ("dun", vec!["顿", "吨", "盾"]),
            ("duo", vec!["多", "度", "躲", "朵"]),
            ("e", vec!["额", "恶", "饿", "俄"]),
            ("en", vec!["恩"]),
            ("er", vec!["二", "儿", "耳", "而"]),
            ("fa", vec!["发", "法", "罚", "乏"]),
            ("fan", vec!["反", "饭", "犯", "范", "繁"]),
            ("fang", vec!["方", "放", "房", "访", "防"]),
            ("fei", vec!["非", "飞", "费", "肥", "废"]),
            ("fen", vec!["分", "份", "粉", "奋", "纷"]),
            ("feng", vec!["风", "封", "丰", "峰", "锋"]),
            ("fo", vec!["佛"]),
            ("fou", vec!["否"]),
            ("fu", vec!["父", "付", "服", "府", "负", "复", "福", "富"]),
            ("ga", vec!["嘎"]),
            ("gai", vec!["改", "该", "盖", "概"]),
            ("gan", vec!["干", "敢", "感", "甘", "肝"]),
            ("gang", vec!["刚", "钢", "港", "岗"]),
            ("gao", vec!["高", "告", "搞", "稿"]),
            ("ge", vec!["个", "各", "歌", "格", "哥"]),
            ("gei", vec!["给"]),
            ("gen", vec!["跟", "根", "更"]),
            ("geng", vec!["更", "耕"]),
            ("gong", vec!["工", "公", "共", "功", "供"]),
            ("gou", vec!["够", "狗", "构", "购"]),
            ("gu", vec!["古", "故", "顾", "骨", "谷"]),
            ("gua", vec!["挂", "瓜", "刮"]),
            ("guai", vec!["怪", "乖", "拐"]),
            ("guan", vec!["关", "观", "管", "官", "馆"]),
            ("guang", vec!["光", "广", "逛"]),
            ("gui", vec!["贵", "规", "鬼", "归", "轨"]),
            ("gun", vec!["滚", "棍"]),
            ("guo", vec!["过", "国", "果", "锅"]),
            ("ha", vec!["哈"]),
            ("hai", vec!["还", "海", "害", "孩"]),
            ("han", vec!["汉", "汗", "寒", "喊", "韩"]),
            ("hang", vec!["行", "航", "杭", "巷"]),
            ("hao", vec!["好", "号", "豪", "毫"]),
            ("he", vec!["和", "合", "河", "何", "核"]),
            ("hei", vec!["黑"]),
            ("hen", vec!["很", "狠", "恨"]),
            ("heng", vec!["横", "恒", "衡"]),
            ("hong", vec!["红", "宏", "洪", "轰"]),
            ("hou", vec!["后", "候", "厚", "喉"]),
            ("hu", vec!["户", "湖", "呼", "虎", "护"]),
            ("hua", vec!["化", "花", "话", "画", "华"]),
            ("huai", vec!["坏", "怀"]),
            ("huan", vec!["换", "还", "环", "欢", "缓"]),
            ("huang", vec!["黄", "皇", "荒", "慌"]),
            ("hui", vec!["会", "回", "灰", "汇", "挥"]),
            ("hun", vec!["混", "婚", "魂", "昏"]),
            ("huo", vec!["或", "活", "火", "货", "获"]),
            (
                "ji",
                vec!["机", "己", "记", "级", "计", "几", "即", "极", "集", "急"],
            ),
            ("jia", vec!["家", "加", "价", "假", "架"]),
            ("jian", vec!["见", "间", "建", "件", "简"]),
            ("jiang", vec!["将", "江", "讲", "强", "奖"]),
            ("jiao", vec!["教", "交", "叫", "脚", "角"]),
            ("jie", vec!["接", "街", "结", "节", "杰"]),
            ("jin", vec!["进", "金", "近", "紧", "尽"]),
            ("jing", vec!["经", "京", "精", "晶", "睛"]),
            ("jiu", vec!["就", "九", "酒", "久", "旧"]),
            ("ju", vec!["局", "居", "举", "巨", "聚"]),
            ("juan", vec!["卷", "捐", "娟", "倦"]),
            ("jue", vec!["觉", "决", "绝", "掘"]),
            ("jun", vec!["军", "君", "均", "俊"]),
            ("ka", vec!["卡", "咖"]),
            ("kai", vec!["开", "凯", "楷"]),
            ("kan", vec!["看", "砍", "坎", "刊"]),
            ("kang", vec!["抗", "康", "扛", "亢"]),
            ("kao", vec!["考", "靠", "烤", "拷"]),
            ("ke", vec!["可", "科", "客", "克", "课"]),
            ("ken", vec!["肯", "恳"]),
            ("keng", vec!["坑"]),
            ("kong", vec!["空", "控", "孔"]),
            ("kou", vec!["口", "扣", "叩"]),
            ("ku", vec!["苦", "哭", "库", "酷"]),
            ("kua", vec!["跨", "夸", "垮"]),
            ("kuai", vec!["快", "块", "筷"]),
            ("kuan", vec!["宽", "款"]),
            ("kuang", vec!["况", "矿", "狂", "框"]),
            ("kun", vec!["困", "昆", "坤"]),
            ("kuo", vec!["扩", "阔", "括"]),
            ("la", vec!["拉", "啦", "辣", "腊"]),
            ("lai", vec!["来", "赖", "莱"]),
            ("lan", vec!["兰", "蓝", "烂", "懒", "拦"]),
            ("lang", vec!["浪", "狼", "郎", "朗"]),
            ("lao", vec!["老", "劳", "牢", "捞"]),
            ("le", vec!["了", "乐", "勒"]),
            ("lei", vec!["累", "类", "雷", "泪"]),
            ("leng", vec!["冷", "愣"]),
            ("li", vec!["里", "力", "理", "利", "立", "离"]),
            ("lia", vec!["俩"]),
            ("lian", vec!["连", "联", "脸", "练", "莲"]),
            ("liang", vec!["两", "亮", "良", "量", "凉"]),
            ("liao", vec!["了", "料", "聊", "辽", "疗"]),
            ("lie", vec!["列", "烈", "裂", "劣"]),
            ("lin", vec!["林", "临", "邻", "淋"]),
            ("ling", vec!["令", "领", "零", "灵", "玲"]),
            ("liu", vec!["流", "六", "留", "刘", "柳"]),
            ("long", vec!["龙", "隆", "弄", "笼"]),
            ("lou", vec!["楼", "漏", "露", "搂"]),
            ("lu", vec!["路", "录", "陆", "露", "卢"]),
            ("lv", vec!["律", "绿", "率", "旅", "虑"]),
            ("luan", vec!["乱", "卵"]),
            ("lue", vec!["略", "掠"]),
            ("lun", vec!["论", "轮", "伦"]),
            ("luo", vec!["落", "罗", "络", "洛"]),
            ("ma", vec!["吗", "妈", "马", "麻", "骂"]),
            ("mai", vec!["买", "卖", "迈", "麦"]),
            ("man", vec!["满", "慢", "曼", "漫"]),
            ("mang", vec!["忙", "芒", "盲", "茫"]),
            ("mao", vec!["毛", "猫", "矛", "茅", "茂"]),
            ("me", vec!["么"]),
            ("mei", vec!["没", "美", "妹", "每", "梅"]),
            ("men", vec!["们", "门", "闷"]),
            ("meng", vec!["梦", "猛", "蒙", "盟"]),
            ("mi", vec!["米", "密", "迷", "秘", "蜜"]),
            ("mian", vec!["面", "棉", "眠", "绵"]),
            ("miao", vec!["秒", "苗", "描", "妙"]),
            ("mie", vec!["灭", "蔑"]),
            ("min", vec!["民", "敏", "名", "闽"]),
            ("ming", vec!["名", "明", "命", "鸣", "铭"]),
            ("miu", vec!["谬"]),
            ("mo", vec!["没", "么", "模", "磨", "摩"]),
            ("mou", vec!["某", "谋"]),
            ("mu", vec!["母", "木", "目", "幕", "牧"]),
            ("na", vec!["那", "拿", "哪", "纳"]),
            ("nai", vec!["奶", "耐", "乃", "奈"]),
            ("nan", vec!["南", "难", "男"]),
            ("nang", vec!["囊"]),
            ("nao", vec!["脑", "恼", "闹"]),
            ("ne", vec!["呢"]),
            ("nei", vec!["内"]),
            ("nen", vec!["嫩"]),
            ("neng", vec!["能"]),
            ("ni", vec!["你", "呢", "泥", "逆", "拟"]),
            ("nian", vec!["年", "念", "粘", "碾"]),
            ("niang", vec!["娘", "酿"]),
            ("niao", vec!["鸟", "尿"]),
            ("nie", vec!["捏", "聂", "孽", "镍"]),
            ("nin", vec!["您"]),
            ("ning", vec!["宁", "凝", "拧", "柠"]),
            ("niu", vec!["牛", "扭", "纽"]),
            ("nong", vec!["农", "浓", "弄"]),
            ("nu", vec!["女", "怒", "努"]),
            ("nv", vec!["女"]),
            ("nuan", vec!["暖"]),
            ("nue", vec!["虐", "疟"]),
            ("nuo", vec!["诺", "挪", "懦"]),
            ("o", vec!["哦"]),
            ("ou", vec!["欧", "偶", "呕", "藕"]),
            ("pa", vec!["怕", "爬", "帕", "趴"]),
            ("pai", vec!["排", "拍", "牌", "派"]),
            ("pan", vec!["判", "盘", "盼", "攀"]),
            ("pang", vec!["旁", "胖", "庞"]),
            ("pao", vec!["跑", "炮", "抛", "泡"]),
            ("pei", vec!["配", "培", "赔", "陪"]),
            ("pen", vec!["盆", "喷"]),
            ("peng", vec!["朋", "鹏", "碰", "捧"]),
            ("pi", vec!["批", "皮", "披", "匹", "劈"]),
            ("pian", vec!["片", "篇", "骗", "偏"]),
            ("piao", vec!["票", "漂", "飘"]),
            ("pie", vec!["撇", "瞥"]),
            ("pin", vec!["品", "拼", "频", "贫"]),
            ("ping", vec!["平", "评", "瓶", "凭"]),
            ("po", vec!["破", "婆", "迫", "坡"]),
            ("pou", vec!["剖"]),
            ("pu", vec!["普", "铺", "朴", "谱"]),
            ("qi", vec!["起", "气", "期", "其", "七"]),
            ("qia", vec!["恰", "洽", "卡"]),
            ("qian", vec!["前", "钱", "千", "签", "牵"]),
            ("qiang", vec!["强", "枪", "墙", "抢"]),
            ("qiao", vec!["桥", "瞧", "巧", "敲"]),
            ("qie", vec!["切", "且", "茄", "怯"]),
            ("qin", vec!["亲", "琴", "勤", "秦", "侵"]),
            ("qing", vec!["请", "清", "情", "青", "轻"]),
            ("qiong", vec!["穷", "琼"]),
            ("qiu", vec!["求", "球", "秋", "丘"]),
            ("qu", vec!["去", "区", "曲", "取", "娶"]),
            ("quan", vec!["全", "权", "圈", "泉", "拳"]),
            ("que", vec!["却", "确", "缺", "雀"]),
            ("qun", vec!["群", "裙"]),
            ("ran", vec!["然", "燃", "染"]),
            ("rang", vec!["让", "嚷", "攘"]),
            ("rao", vec!["绕", "扰", "饶"]),
            ("re", vec!["热", "惹"]),
            ("ren", vec!["人", "认", "任", "仁", "忍"]),
            ("reng", vec!["仍", "扔"]),
            ("ri", vec!["日"]),
            ("rong", vec!["容", "荣", "融", "绒"]),
            ("rou", vec!["肉", "柔", "揉"]),
            ("ru", vec!["入", "如", "儒", "乳"]),
            ("ruan", vec!["软", "阮"]),
            ("rui", vec!["瑞", "锐", "睿"]),
            ("run", vec!["润", "闰"]),
            ("ruo", vec!["若", "弱"]),
            ("sa", vec!["撒", "洒", "萨"]),
            ("sai", vec!["赛", "塞", "腮"]),
            ("san", vec!["三", "散", "伞"]),
            ("sang", vec!["丧", "桑", "嗓"]),
            ("sao", vec!["扫", "嫂", "骚"]),
            ("se", vec!["色", "塞", "瑟"]),
            ("sen", vec!["森"]),
            ("seng", vec!["僧"]),
            ("sha", vec!["杀", "沙", "纱", "傻"]),
            ("shai", vec!["晒", "筛"]),
            ("shan", vec!["山", "善", "闪", "衫"]),
            ("shang", vec!["上", "商", "伤", "赏"]),
            ("shao", vec!["少", "烧", "绍", "稍"]),
            ("she", vec!["设", "社", "射", "涉"]),
            ("shen", vec!["深", "身", "神", "什", "申"]),
            ("sheng", vec!["生", "声", "省", "胜", "升"]),
            ("shi", vec!["是", "时", "事", "实", "十"]),
            ("shou", vec!["手", "首", "受", "收"]),
            ("shu", vec!["书", "数", "树", "术", "输"]),
            ("shua", vec!["刷", "耍"]),
            ("shuai", vec!["帅", "率", "摔"]),
            ("shuan", vec!["栓", "拴"]),
            ("shuang", vec!["双", "爽", "霜"]),
            ("shui", vec!["水", "说", "睡", "税"]),
            ("shun", vec!["顺", "瞬", "舜"]),
            ("shuo", vec!["说", "硕", "朔"]),
            ("si", vec!["四", "死", "思", "司", "斯"]),
            ("song", vec!["送", "松", "宋", "颂"]),
            ("sou", vec!["搜", "艘"]),
            ("su", vec!["速", "素", "苏", "诉"]),
            ("suan", vec!["算", "酸", "蒜"]),
            ("sui", vec!["岁", "随", "碎", "虽"]),
            ("sun", vec!["孙", "损", "笋"]),
            ("suo", vec!["所", "锁", "索", "缩"]),
            ("ta", vec!["他", "她", "它", "塔"]),
            ("tai", vec!["太", "台", "态", "泰"]),
            ("tan", vec!["谈", "探", "坦", "摊"]),
            ("tang", vec!["堂", "唐", "糖", "躺"]),
            ("tao", vec!["套", "逃", "桃", "陶"]),
            ("te", vec!["特"]),
            ("teng", vec!["疼", "腾", "藤"]),
            ("ti", vec!["提", "题", "体", "替"]),
            ("tian", vec!["天", "田", "填", "甜"]),
            ("tiao", vec!["条", "调", "跳", "挑"]),
            ("tie", vec!["铁", "贴", "帖"]),
            ("ting", vec!["听", "停", "庭", "厅"]),
            ("tong", vec!["同", "通", "统", "痛"]),
            ("tou", vec!["头", "投", "透", "偷"]),
            ("tu", vec!["土", "图", "突", "途"]),
            ("tuan", vec!["团", "湍"]),
            ("tui", vec!["推", "退", "腿"]),
            ("tun", vec!["吞", "屯", "囤"]),
            ("tuo", vec!["脱", "拖", "托", "驼"]),
            ("wa", vec!["挖", "哇", "蛙", "娃"]),
            ("wai", vec!["外", "歪"]),
            ("wan", vec!["完", "晚", "万", "玩"]),
            ("wang", vec!["王", "网", "往", "忘"]),
            ("wei", vec!["为", "位", "未", "围", "委"]),
            ("wen", vec!["问", "文", "闻", "温"]),
            ("weng", vec!["翁", "嗡"]),
            ("wo", vec!["我", "窝", "握", "卧"]),
            ("wu", vec!["无", "五", "物", "务", "武"]),
            ("xi", vec!["系", "西", "息", "希", "席"]),
            ("xia", vec!["下", "夏", "吓", "虾"]),
            ("xian", vec!["先", "现", "线", "限", "显"]),
            ("xiang", vec!["想", "向", "相", "乡", "香"]),
            ("xiao", vec!["小", "笑", "校", "消", "效"]),
            ("xie", vec!["写", "些", "谢", "鞋", "协"]),
            ("xin", vec!["新", "心", "信", "欣", "辛"]),
            ("xing", vec!["行", "星", "性", "形", "姓"]),
            ("xiong", vec!["雄", "兄", "胸", "凶"]),
            ("xiu", vec!["修", "休", "秀", "袖"]),
            ("xu", vec!["需", "须", "续", "许", "序"]),
            ("xuan", vec!["选", "宣", "县", "悬"]),
            ("xue", vec!["学", "血", "雪", "穴"]),
            ("xun", vec!["寻", "讯", "训", "迅"]),
            ("ya", vec!["呀", "压", "牙", "亚"]),
            ("yan", vec!["言", "眼", "研", "严", "演"]),
            ("yang", vec!["样", "洋", "阳", "扬", "氧"]),
            ("yao", vec!["要", "药", "摇", "腰", "遥"]),
            ("ye", vec!["也", "业", "夜", "叶", "爷"]),
            ("yi", vec!["一", "以", "意", "义", "已"]),
            ("yin", vec!["因", "引", "音", "银", "印"]),
            ("ying", vec!["应", "英", "影", "营", "迎"]),
            ("yo", vec!["哟"]),
            ("yong", vec!["用", "永", "拥", "勇"]),
            ("you", vec!["有", "又", "由", "友", "右"]),
            ("yu", vec!["与", "于", "语", "雨", "玉"]),
            ("yuan", vec!["元", "原", "员", "源", "远"]),
            ("yue", vec!["月", "约", "越", "跃"]),
            ("yun", vec!["云", "运", "员", "韵"]),
            ("za", vec!["杂", "砸"]),
            ("zai", vec!["在", "再", "载", "灾"]),
            ("zan", vec!["咱", "赞", "暂"]),
            ("zang", vec!["藏", "脏", "葬"]),
            ("zao", vec!["早", "造", "遭", "燥"]),
            ("ze", vec!["则", "责", "择", "泽"]),
            ("zei", vec!["贼"]),
            ("zen", vec!["怎"]),
            ("zeng", vec!["增", "曾", "赠"]),
            ("zha", vec!["扎", "炸", "查", "渣"]),
            ("zhai", vec!["债", "宅", "寨", "窄"]),
            ("zhan", vec!["战", "站", "占", "展"]),
            ("zhang", vec!["长", "张", "章", "掌"]),
            ("zhao", vec!["找", "照", "招", "赵"]),
            ("zhe", vec!["这", "着", "者", "折"]),
            ("zhen", vec!["真", "镇", "针", "阵"]),
            ("zheng", vec!["正", "政", "证", "整"]),
            ("zhi", vec!["之", "知", "制", "只", "至"]),
            ("zhong", vec!["中", "种", "重", "众"]),
            ("zhou", vec!["周", "州", "洲", "舟"]),
            ("zhu", vec!["主", "住", "注", "助"]),
            ("zhua", vec!["抓"]),
            ("zhuai", vec!["拽"]),
            ("zhuan", vec!["专", "转", "传", "赚"]),
            ("zhuang", vec!["装", "状", "庄", "撞"]),
            ("zhui", vec!["追", "坠", "缀"]),
            ("zhun", vec!["准"]),
            ("zhuo", vec!["着", "桌", "捉"]),
            ("zi", vec!["子", "自", "字", "资"]),
            ("zong", vec!["总", "纵", "宗", "综"]),
            ("zou", vec!["走", "奏", "邹"]),
            ("zu", vec!["组", "足", "族", "租"]),
            ("zuan", vec!["钻", "纂"]),
            ("zui", vec!["最", "嘴", "罪"]),
            ("zun", vec!["尊", "遵"]),
            ("zuo", vec!["作", "做", "座", "左"]),
        ];

        for (pinyin, words) in default_words {
            let entries: Vec<DictEntry> = words
                .iter()
                .enumerate()
                .map(|(i, word)| DictEntry {
                    word: word.to_string(),
                    pinyin: pinyin.to_string(),
                    frequency: (100 - i as u64).max(1),
                    last_used: None,
                })
                .collect();
            self.system_dict.insert(pinyin.to_string(), entries);
        }
    }

    /// Lookup candidates for a pinyin string
    pub fn lookup(&self, pinyin: &str) -> Vec<DictEntry> {
        let mut results = Vec::new();

        // First check user dictionary (higher priority)
        {
            let user_dict = self.user_dict.read();
            if let Some(entries) = user_dict.get(pinyin) {
                results.extend(entries.clone());
            }
        }

        // Then check system dictionary
        if let Some(entries) = self.system_dict.get(pinyin) {
            for entry in entries {
                // Check if this word exists in user dict
                if let Some(user_entry) = results.iter_mut().find(|e| e.word == entry.word) {
                    // Merge frequencies: use user frequency + system frequency as boost
                    user_entry.frequency += entry.frequency;
                } else {
                    // Add system entry if not in user dict
                    results.push(entry.clone());
                }
            }
        }

        // Sort by frequency
        results.sort_by(|a, b| b.frequency.cmp(&a.frequency));

        results
    }

    /// Update word frequency in user dictionary
    pub fn update_frequency(&self, pinyin: &str, word: &str) {
        if !self.config.enable_learning {
            return;
        }

        let mut user_dict = self.user_dict.write();

        if let Some(entries) = user_dict.get_mut(pinyin) {
            if let Some(entry) = entries.iter_mut().find(|e| e.word == word) {
                entry.frequency = entry.frequency.saturating_add(1);
                entry.last_used = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                );
                return;
            }
        }

        // Add new entry
        let entry = DictEntry {
            word: word.to_string(),
            pinyin: pinyin.to_string(),
            frequency: 1,
            last_used: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            ),
        };

        user_dict
            .entry(pinyin.to_string())
            .or_insert_with(Vec::new)
            .push(entry);
    }

    /// Save user dictionary to file
    pub fn save_user_dictionary(&self) -> std::io::Result<()> {
        let path = expand_path(&self.config.user_dictionary);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        let mut writer = BufWriter::new(file);
        let user_dict = self.user_dict.read();

        for (_pinyin, entries) in user_dict.iter() {
            for entry in entries {
                writeln!(
                    writer,
                    "{}\t{}\t{}\t{}",
                    entry.word,
                    entry.pinyin,
                    entry.frequency,
                    entry.last_used.unwrap_or(0)
                )?;
            }
        }

        Ok(())
    }
}

/// Expand path with ~ support
fn expand_path(path: &str) -> PathBuf {
    if path.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(path.replacen('~', &home, 1))
    } else {
        PathBuf::from(path)
    }
}

/// Load dictionary from file
fn load_dictionary_file(path: &Path) -> std::io::Result<HashMap<String, Vec<DictEntry>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut dict: HashMap<String, Vec<DictEntry>> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split('\t').collect();

        if parts.len() >= 2 {
            let word = parts[0].to_string();
            let pinyin = parts[1].to_string();
            let frequency = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
            let last_used = parts.get(3).and_then(|s| s.parse().ok());

            let entry = DictEntry {
                word,
                pinyin: pinyin.clone(),
                frequency,
                last_used,
            };

            dict.entry(pinyin).or_insert_with(Vec::new).push(entry);
        }
    }

    Ok(dict)
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new(DictionaryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_lookup() {
        let dict = Dictionary::default();
        let results = dict.lookup("ni");
        assert!(!results.is_empty());
        assert!(results.iter().any(|e| e.word == "你"));
    }

    #[test]
    fn test_frequency_update() {
        let dict = Dictionary::default();
        dict.update_frequency("ni", "你");

        let results = dict.lookup("ni");
        let ni_entry = results.iter().find(|e| e.word == "你");
        assert!(ni_entry.is_some());
    }
}
