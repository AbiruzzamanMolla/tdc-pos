import { defineStore } from 'pinia';

export const useI18nStore = defineStore('i18n', {
  state: () => ({
    locale: localStorage.getItem('locale') || 'en',
    translations: {
      en: {
        dashboard: 'Dashboard',
        executive_dashboard: 'Executive Dashboard',
        precision_analytics: 'Precision Business Analytics',
        refresh_data: 'Refresh Data',
        inventory_valuation: 'Inventory Valuation',
        current_assets: 'Current Assets',
        lifetime_profit: 'Lifetime Profit',
        net_accrued: 'Net Accrued',
        active_orders: 'Active Orders',
        low_stock_items: 'Low Stock Items',
        operational_performance: 'Operational performance',
        temporal_visualization: 'Temporal Data Visualization',
        sales: 'Sales',
        purchases: 'Purchases',
        profit: 'Profit',
        today: 'Today',
        this_month: 'This Month',
        this_year: 'This Year',
        procurement_today: 'Procurement Today',
        monthly_margin: 'Monthly Margin',
        annual_profit: 'Annual Profit',
        real_time_analysis: 'Real-time margin analysis',
        snapshot_accuracy: 'Snapshot based accuracy',
        fiscal_performance: 'Fiscal year performance',
        operational_shortcuts: 'Operational Shortcuts',
        checkout: 'Checkout',
        stock: 'Stock',
        receive: 'Receive',
        stats: 'Stats',
        system_integrity: 'System Integrity',
        catalog_size: 'Catalog Size',
        auth_status: 'Auth Status',

        last_updated: 'Last Updated',
        locale_name: 'English',
      },
      bn: {
        dashboard: 'ড্যাশবোর্ড',
        executive_dashboard: 'এক্সিকিউটিভ ড্যাশবোর্ড',
        precision_analytics: 'বিজনেসের সঠিক বিশ্লেষণ',
        refresh_data: 'ডাটা রিফ্রেশ করুন',
        inventory_valuation: 'ইনভেন্টরি মূল্যায়ন',
        current_assets: 'বর্তমান সম্পদ',
        lifetime_profit: 'মোট লাভ',
        net_accrued: 'নিট অর্জিত',
        active_orders: 'সক্রিয় অর্ডার',
        low_stock_items: 'কম স্টকের পণ্য',
        operational_performance: 'ব্যবসায়িক কর্মক্ষমতা',
        temporal_visualization: 'সময়ের ভিত্তিতে ডাটা বিশ্লেষণ',
        sales: 'বিক্রয়',
        purchases: 'ক্রয়',
        profit: 'লাভ',
        today: 'আজ',
        this_month: 'এই মাস',
        this_year: 'এই বছর',
        procurement_today: 'আজকের ক্রয়',
        monthly_margin: 'মাসিক মার্জিন',
        annual_profit: 'বার্ষিক লাভ',
        real_time_analysis: 'রিয়েল-টাইম মার্জিন বিশ্লেষণ',
        snapshot_accuracy: 'স্ন্যাপশট ভিত্তিক নির্ভুলতা',
        fiscal_performance: 'অর্থবছরের কর্মক্ষমতা',
        operational_shortcuts: 'প্রয়োজনীয় শর্টকাট',
        checkout: 'চেকআউট',
        stock: 'স্টক',
        receive: 'পণ্য গ্রহণ',
        stats: 'পরিসংখ্যান',
        system_integrity: 'সিস্টেম ইন্টেগ্রিটি',
        catalog_size: 'ক্যাটালগ সাইজ',
        auth_status: 'ইউজার রোল',
        last_updated: 'সর্বশেষ আপডেট',
        locale_name: 'বাংলা',
      }
    }
  }),
  getters: {
    t: (state) => (key) => {
      const localeTranslations = state.translations[state.locale] || state.translations['en'];
      return localeTranslations[key] || key;
    }
  },
  actions: {
    setLocale(newLocale) {
      this.locale = newLocale;
      localStorage.setItem('locale', newLocale);
    },
    toggleLocale() {
      const newLocale = this.locale === 'en' ? 'bn' : 'en';
      this.setLocale(newLocale);
    }
  }
});
